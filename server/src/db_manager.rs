use json_patch::{Patch, PatchOperation};
use log::{debug, error, info};
use rust_rocksdb::backup::{BackupEngine, BackupEngineInfo, BackupEngineOptions, RestoreOptions};
use rust_rocksdb::{Cache, ColumnFamilyDescriptor, DBCompressionType, DBWithThreadMode, Env, MergeOperands, MultiThreaded, Options, Transaction, TransactionDB, TransactionDBOptions, TransactionOptions, WriteBatchWithTransaction, WriteOptions};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::time::Duration;

pub type DbInstance = Arc<RwLock<Option<DBWithThreadMode<MultiThreaded>>>>;

pub fn json_merge(
    _new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    let mut doc: Value = if let Some(val) = existing_val {
        serde_json::from_slice(val).unwrap_or(Value::Array(vec![]))
    } else {
        Value::Array(vec![])
    };

    for op in operands {
        if let Ok(patch) = serde_json::from_slice::<Vec<PatchOperation>>(op) {
            let patch = Patch(patch);
            if let Err(e) = json_patch::patch(&mut doc, &patch) {
                error!("Failed to apply patch: {:?}", e);
            }
        } else {
            error!("Failed to deserialize operand");
        }
    }

    match serde_json::to_vec(&doc) {
        Ok(bytes) => Some(bytes),
        Err(e) => {
            error!("Failed to serialize JSON: {:?}", e);
            None
        }
    }
}

fn create_transaction(transaction_db: &Arc<TransactionDB>) -> Transaction<'static, TransactionDB> {
    let txn_opts = TransactionOptions::default();
    let write_opts = WriteOptions::default();
    unsafe {
        std::mem::transmute::<Transaction<TransactionDB>, Transaction<'static, TransactionDB>>(
            transaction_db.transaction_opt(&write_opts, &txn_opts),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackupInfo {
    timestamp: i64,
    backup_id: u32,
    size: u64,
    num_files: u32,
}

impl From<BackupEngineInfo> for BackupInfo {
    fn from(info: BackupEngineInfo) -> Self {
        BackupInfo {
            timestamp: info.timestamp,
            backup_id: info.backup_id,
            size: info.size,
            num_files: info.num_files,
        }
    }
}

pub struct RocksDBManager {
    pub db: DbInstance,
    pub db_path: String,
    write_batch: Mutex<Option<WriteBatchWithTransaction<false>>>,
    iterators: Mutex<HashMap<usize, (Vec<u8>, rust_rocksdb::Direction)>>,
    iterator_id_counter: AtomicUsize,
    txn_db: Mutex<Option<Arc<TransactionDB>>>,
    transaction: Mutex<Option<Transaction<'static, TransactionDB>>>,
    condvar: Condvar,
}

impl RocksDBManager {
    pub fn new(db_path: &str, ttl_secs: Option<u64>) -> Result<Self, String> {
        info!(
            "Initializing RocksDBManager with db_path: {}, ttl_secs: {:?}",
            db_path, ttl_secs
        );

        let cache = Cache::new_lru_cache(512 * 1024 * 1024); // 512 MB
        let mut opts = Options::default();
        opts.set_row_cache(&cache);
        opts.create_if_missing(true);
        opts.set_merge_operator_associative("json_merge", json_merge);
        opts.increase_parallelism(num_cpus::get() as i32);
        opts.optimize_level_style_compaction(512 * 1024 * 1024); // 512 MB
        opts.set_compression_type(DBCompressionType::Snappy);
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64 MB
        opts.set_max_write_buffer_number(3);
        opts.set_min_write_buffer_number_to_merge(1);
        opts.set_max_open_files(1000);

        let cf_names = DBWithThreadMode::<MultiThreaded>::list_cf(&opts, db_path)
            .unwrap_or(vec!["default".to_string()]);
        let cf_descriptors: Vec<ColumnFamilyDescriptor> = cf_names
            .iter()
            .map(|name| {
                let mut cf_opts = Options::default();
                cf_opts.set_merge_operator_associative("json_merge", json_merge);
                ColumnFamilyDescriptor::new(name, cf_opts)
            })
            .collect();

        let db = match ttl_secs {
            Some(ttl) => {
                let duration = Duration::from_secs(ttl);
                DBWithThreadMode::<MultiThreaded>::open_cf_descriptors_with_ttl(
                    &opts,
                    db_path,
                    cf_descriptors,
                    duration,
                )
                    .map_err(|e| e.to_string())?
            }
            None => DBWithThreadMode::<MultiThreaded>::open_cf_descriptors(
                &opts,
                db_path,
                cf_descriptors,
            )
                .map_err(|e| e.to_string())?,
        };

        let db = Arc::new(RwLock::new(Some(db)));

        let iterators = Mutex::new(HashMap::new());
        let iterator_id_counter = AtomicUsize::new(0);

        info!("RocksDBManager initialized successfully");

        Ok(RocksDBManager {
            db,
            db_path: db_path.to_string(),
            write_batch: Mutex::new(Some(WriteBatchWithTransaction::default())),
            iterators,
            iterator_id_counter,
            txn_db: Mutex::new(None),
            transaction: Mutex::new(None),
            condvar: Condvar::new(),
        })
    }

    pub fn begin_transaction(&self) -> Result<(), String> {
        info!("Beginning new transaction");

        let mut txn_db_lock = self.txn_db.lock().unwrap();
        let mut transaction_lock = self.transaction.lock().unwrap();

        while txn_db_lock.is_some() || transaction_lock.is_some() {
            txn_db_lock = self.condvar.wait(txn_db_lock).unwrap();
            transaction_lock = self.condvar.wait(transaction_lock).unwrap();
        }

        self.close()?;

        let txn_db_opts = TransactionDBOptions::default();
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(1000);
        opts.set_log_level(rust_rocksdb::LogLevel::Warn);

        let cf_names = DBWithThreadMode::<MultiThreaded>::list_cf(&opts, &self.db_path)
            .unwrap_or(vec!["default".to_string()]);
        let cf_descriptors: Vec<ColumnFamilyDescriptor> = cf_names
            .iter()
            .map(|name| {
                let mut cf_opts = Options::default();
                cf_opts.set_merge_operator_associative("json_merge", json_merge);
                ColumnFamilyDescriptor::new(name, cf_opts)
            })
            .collect();

        let transaction_db: TransactionDB =
            TransactionDB::open_cf_descriptors(&opts, &txn_db_opts, &self.db_path, cf_descriptors).map_err(|e| e.to_string())?;

        let transaction_db = Arc::new(transaction_db);
        let transaction = create_transaction(&transaction_db);

        *txn_db_lock = Some(transaction_db);
        *transaction_lock = Some(transaction);

        Ok(())
    }

    pub fn commit_transaction(&self) -> Result<(), String> {
        info!("Committing transaction");
        let mut transaction_lock = self.transaction.lock().unwrap();
        let mut txn_db_lock = self.txn_db.lock().unwrap();

        if transaction_lock.is_none() {
            return Err("No active transaction to commit".to_string());
        }

        let txn = transaction_lock.take().unwrap();
        let result = txn.commit().map_err(|e| e.to_string());

        *txn_db_lock = None;
        *transaction_lock = None;
        self.condvar.notify_all();

        if result.is_ok() {
            self.reopen().map_err(|e| e.to_string())?;
        }

        result
    }

    pub fn rollback_transaction(&self) -> Result<(), String> {
        info!("Rolling back transaction");
        let mut transaction_lock = self.transaction.lock().unwrap();
        let mut txn_db_lock = self.txn_db.lock().unwrap();

        if transaction_lock.is_none() {
            return Err("No active transaction to rollback".to_string());
        }

        let txn = transaction_lock.take().unwrap();
        let _ = txn.rollback().map_err(|e| e.to_string());
        let result = txn.commit().map_err(|e| e.to_string());

        *txn_db_lock = None;
        *transaction_lock = None;
        self.condvar.notify_all();

        if result.is_ok() {
            self.reopen().map_err(|e| e.to_string())?;
        }

        result
    }

    pub fn put(
        &self,
        key: String,
        value: String,
        cf_name: Option<String>,
        txn: Option<bool>,
    ) -> Result<(), String> {
        debug!(
            "Putting key: {}, value: {}, cf_name: {:?}, txn: {:?}",
            key, value, cf_name, txn
        );

        let mut transaction_lock = self.transaction.lock().unwrap();

        if let Some(txn) = txn {
            if txn && transaction_lock.is_some() {
                let txn = transaction_lock.as_ref().unwrap();
                let txn_db_lock = self.txn_db.lock().unwrap();
                let txn_db = txn_db_lock.as_ref().unwrap();

                match cf_name {
                    Some(cf_name) => {
                        let cf = txn_db.cf_handle(&cf_name).ok_or("Column family not found")?;
                        txn.put_cf(&cf, key.as_bytes(), value.as_bytes())
                            .map_err(|e| e.to_string())
                    }
                    None => txn
                        .put(key.as_bytes(), value.as_bytes())
                        .map_err(|e| e.to_string()),
                }
            } else {
                Err("Transaction is not active or txn is false".to_string())
            }
        } else if transaction_lock.is_some() {
            while transaction_lock.is_some() {
                transaction_lock = self.condvar.wait(transaction_lock).unwrap(); // Wait for the transaction to complete
            }
            self.merge(key, value, cf_name, txn) // Retry the operation
        } else {
            let db = self.db.read().unwrap();
            if db.is_none() {
                return Err("Database is not open".to_string());
            }
            let db = db.as_ref().unwrap();

            match cf_name {
                Some(cf_name) => {
                    let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    db.put_cf(&cf, key.as_bytes(), value.as_bytes())
                        .map_err(|e| e.to_string())
                }
                None => db
                    .put(key.as_bytes(), value.as_bytes())
                    .map_err(|e| e.to_string()),
            }
        }
    }

    pub fn get(
        &self,
        key: String,
        cf_name: Option<String>,
        default: Option<String>,
        txn: Option<bool>,
    ) -> Result<Option<String>, String> {
        debug!(
            "Getting key: {}, cf_name: {:?}, default: {:?}, txn: {:?}",
            key, cf_name, default, txn
        );

        let mut transaction_lock = self.transaction.lock().unwrap();

        if let Some(txn) = txn {
            if txn && transaction_lock.is_some() {
                let txn = transaction_lock.as_ref().unwrap();
                let txn_db_lock = self.txn_db.lock().unwrap();
                let txn_db = txn_db_lock.as_ref().unwrap();

                match cf_name {
                    Some(cf_name) => {
                        let cf = txn_db.cf_handle(&cf_name).ok_or("Column family not found")?;
                        match txn.get_cf(&cf, key.as_bytes()) {
                            Ok(Some(value)) => {
                                Ok(Some(String::from_utf8(value).map_err(|e| e.to_string())?))
                            }
                            Ok(None) => Ok(default),
                            Err(e) => Err(e.to_string()),
                        }
                    }
                    None => match txn.get(key.as_bytes()) {
                        Ok(Some(value)) => {
                            Ok(Some(String::from_utf8(value).map_err(|e| e.to_string())?))
                        }
                        Ok(None) => Ok(default),
                        Err(e) => Err(e.to_string()),
                    },
                }
            } else {
                Err("Transaction is not active or txn is false".to_string())
            }
        } else if transaction_lock.is_some() {
            while transaction_lock.is_some() {
                transaction_lock = self.condvar.wait(transaction_lock).unwrap(); // Wait for the transaction to complete
            }
            self.get(key, cf_name, default, txn) // Retry the operation
        } else {
            let db = self.db.read().unwrap();
            if db.is_none() {
                return Err("Database is not open".to_string());
            }
            let db = db.as_ref().unwrap();

            match cf_name {
                Some(cf_name) => {
                    let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    match db.get_cf(&cf, key.as_bytes()) {
                        Ok(Some(value)) => {
                            Ok(Some(String::from_utf8(value).map_err(|e| e.to_string())?))
                        }
                        Ok(None) => Ok(default),
                        Err(e) => Err(e.to_string()),
                    }
                }
                None => match db.get(key.as_bytes()) {
                    Ok(Some(value)) => {
                        Ok(Some(String::from_utf8(value).map_err(|e| e.to_string())?))
                    }
                    Ok(None) => Ok(default),
                    Err(e) => Err(e.to_string()),
                },
            }
        }
    }

    pub fn delete(
        &self,
        key: String,
        cf_name: Option<String>,
        txn: Option<bool>,
    ) -> Result<(), String> {
        debug!(
            "Deleting key: {}, cf_name: {:?}, txn: {:?}",
            key, cf_name, txn
        );

        let mut transaction_lock = self.transaction.lock().unwrap();

        if let Some(txn) = txn {
            if txn && transaction_lock.is_some() {
                let txn = transaction_lock.as_ref().unwrap();
                let txn_db_lock = self.txn_db.lock().unwrap();
                let txn_db = txn_db_lock.as_ref().unwrap();

                match cf_name {
                    Some(cf_name) => {
                        let cf = txn_db.cf_handle(&cf_name).ok_or("Column family not found")?;
                        txn.delete_cf(&cf, key.as_bytes())
                            .map_err(|e| e.to_string())
                    }
                    None => txn.delete(key.as_bytes()).map_err(|e| e.to_string()),
                }
            } else {
                Err("Transaction is not active or txn is false".to_string())
            }
        } else if transaction_lock.is_some() {
            while transaction_lock.is_some() {
                transaction_lock = self.condvar.wait(transaction_lock).unwrap(); // Wait for the transaction to complete
            }
            self.delete(key, cf_name, txn) // Retry the operation
        } else {
            let db = self.db.read().unwrap();
            if db.is_none() {
                return Err("Database is not open".to_string());
            }
            let db = db.as_ref().unwrap();
            match cf_name {
                Some(cf_name) => {
                    let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    db.delete_cf(&cf, key.as_bytes()).map_err(|e| e.to_string())
                }
                None => db.delete(key.as_bytes()).map_err(|e| e.to_string()),
            }
        }
    }

    pub fn merge(
        &self,
        key: String,
        value: String,
        cf_name: Option<String>,
        txn: Option<bool>,
    ) -> Result<(), String> {
        debug!(
            "Merging key: {}, value: {}, cf_name: {:?}, txn: {:?}",
            key, value, cf_name, txn
        );

        let mut transaction_lock = self.transaction.lock().unwrap();

        if let Some(txn) = txn {
            if txn && transaction_lock.is_some() {
                let txn = transaction_lock.as_ref().unwrap();
                let txn_db_lock = self.txn_db.lock().unwrap();
                let txn_db = txn_db_lock.as_ref().unwrap();

                match cf_name {
                    Some(cf_name) => {
                        let cf = txn_db.cf_handle(&cf_name).ok_or("Column family not found")?;
                        txn.merge_cf(&cf, key.as_bytes(), value.as_bytes())
                            .map_err(|e| e.to_string())
                    }
                    None => txn
                        .merge(key.as_bytes(), value.as_bytes())
                        .map_err(|e| e.to_string()),
                }
            } else {
                Err("Transaction is not active or txn is false".to_string())
            }
        } else if transaction_lock.is_some() {
            while transaction_lock.is_some() {
                transaction_lock = self.condvar.wait(transaction_lock).unwrap(); // Wait for the transaction to complete
            }
            self.delete(key, cf_name, txn) // Retry the operation
        } else {
            let db = self.db.read().unwrap();
            if db.is_none() {
                return Err("Database is not open".to_string());
            }
            let db = db.as_ref().unwrap();
            match cf_name {
                Some(cf_name) => {
                    let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    db.merge_cf(&cf, key.as_bytes(), value.as_bytes())
                        .map_err(|e| e.to_string())
                }
                None => db
                    .merge(key.as_bytes(), value.as_bytes())
                    .map_err(|e| e.to_string()),
            }
        }
    }

    pub fn get_property(
        &self,
        property: String,
        cf_name: Option<String>,
    ) -> Result<Option<String>, String> {
        debug!("get property with id: {}, cf_name: {:?}", property, cf_name);
        let db = self.db.read().unwrap();
        if let Some(ref db) = *db {
            match cf_name {
                Some(cf_name) => {
                    let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    match db.property_value_cf(&cf, &property) {
                        Ok(Some(value)) => Ok(Some(value)),
                        Ok(None) => Ok(None),
                        Err(e) => Err(e.to_string().into()),
                    }
                }
                None => match db.property_value(&property) {
                    Ok(Some(value)) => Ok(Some(value)),
                    Ok(None) => Ok(None),
                    Err(e) => Err(e.to_string().into()),
                },
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn get_all(&self, query: Option<String>) -> Result<Vec<String>, String> {
        debug!("Get all keys with query: {:?}", query);
        let db = self.db.read().unwrap();
        if let Some(ref db) = *db {
            let iter = db.iterator(rust_rocksdb::IteratorMode::Start);

            let keys: Vec<String> = iter
                .filter_map(|result| match result {
                    Ok((key, value)) => {
                        let key_str = String::from_utf8(key.to_vec()).unwrap();
                        let value_str = String::from_utf8(value.to_vec()).unwrap();
                        if let Some(ref q) = query {
                            if key_str.contains(q) || value_str.contains(q) {
                                Some(key_str)
                            } else {
                                None
                            }
                        } else {
                            Some(key_str)
                        }
                    }
                    Err(_) => None,
                })
                .collect();

            debug!("Get all result: {:?}", keys);
            Ok(keys)
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn get_keys(
        &self,
        start: usize,
        limit: usize,
        query: Option<String>,
    ) -> Result<Vec<String>, String> {
        debug!(
            "Get keys with start: {}, limit: {}, query: {:?}",
            start, limit, query
        );
        let mut keys = self.get_all(query)?;
        keys = keys.into_iter().skip(start).take(limit).collect();
        debug!("Get keys result: {:?}", keys);
        Ok(keys)
    }

    pub fn close(&self) -> Result<(), String> {
        info!("Closing database");
        let mut db_lock = self.db.write().unwrap();
        *db_lock = None;
        Ok(())
    }

    pub fn reopen(&self) -> Result<(), String> {
        info!("Reopening database with db_path: {}", self.db_path);

        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_merge_operator_associative("json_merge", json_merge);

        let cf_names = DBWithThreadMode::<MultiThreaded>::list_cf(&opts, &self.db_path)
            .unwrap_or(vec!["default".to_string()]);
        let cf_descriptors: Vec<ColumnFamilyDescriptor> = cf_names
            .iter()
            .map(|name| {
                let mut cf_opts = Options::default();
                cf_opts.set_merge_operator_associative("json_merge", json_merge);
                ColumnFamilyDescriptor::new(name, cf_opts)
            })
            .collect();

        let new_db = DBWithThreadMode::<MultiThreaded>::open_cf_descriptors(
            &opts,
            &self.db_path,
            cf_descriptors,
        )
            .map_err(|e| e.to_string())?;
        let mut db_lock = self.db.write().unwrap();
        *db_lock = Some(new_db);

        info!("Database reopened successfully");
        Ok(())
    }

    pub fn reload(&self) -> Result<(), String> {
        info!("Reloading database");
        self.close().unwrap();
        self.reopen().unwrap();

        info!("Database reloaded successfully");
        Ok(())
    }

    pub fn list_column_families(&self) -> Result<Vec<String>, String> {
        debug!("Listing column families for path: {}", self.db_path.clone());
        let opts = Options::default();
        let result =
            DBWithThreadMode::<MultiThreaded>::list_cf(&opts, self.db_path.clone()).map_err(|e| e.to_string());
        debug!("List column families result: {:?}", result);
        result
    }

    pub fn create_column_family(&self, cf_name: String) -> Result<(), String> {
        info!("Creating column family: {}", cf_name);
        let mut db = self.db.write().unwrap();
        if let Some(ref mut db) = *db {
            let cf_exists = db.cf_handle(&cf_name).is_some();
            let result = if cf_exists {
                Ok(())
            } else {
                let mut opts = Options::default();
                opts.set_merge_operator_associative("json_merge", json_merge);
                db.create_cf(&cf_name, &opts).map_err(|e| e.to_string())
            };
            debug!("Create column family result: {:?}", result);
            result
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn drop_column_family(&self, cf_name: String) -> Result<(), String> {
        info!("Dropping column family: {}", cf_name);
        let mut db = self.db.write().unwrap();
        if let Some(ref mut db) = *db {
            let cf_exists = db.cf_handle(&cf_name).is_some();
            let result = if !cf_exists {
                Ok(())
            } else {
                db.drop_cf(&cf_name).map_err(|e| e.to_string())
            };
            debug!("Drop column family result: {:?}", result);
            result
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn compact_range(
        &self,
        start: Option<String>,
        end: Option<String>,
        cf_name: Option<String>,
    ) -> Result<(), String> {
        debug!(
            "Compacting range with start: {:?}, end: {:?}, cf_name: {:?}",
            start, end, cf_name
        );
        let db = self.db.read().unwrap();
        if let Some(ref db) = *db {
            let result = match cf_name {
                Some(cf_name) => {
                    let cf = db.cf_handle(&cf_name).unwrap();
                    db.compact_range_cf(&cf, start.as_deref(), end.as_deref());
                    Ok(())
                }
                None => {
                    db.compact_range(start.as_deref(), end.as_deref());
                    Ok(())
                }
            };
            debug!("Compact range result: {:?}", result);
            result
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn write_batch_put(
        &self,
        key: String,
        value: String,
        cf_name: Option<String>,
    ) -> Result<(), String> {
        debug!(
            "Write batch put with key: {}, value: {}, cf_name: {:?}",
            key, value, cf_name
        );
        let db = self.db.read().unwrap();
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref db) = *db {
            let result = if let Some(ref mut wb) = *batch {
                match cf_name {
                    Some(cf_name) => {
                        let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                        wb.put_cf(&cf, key.as_bytes(), value.as_bytes());
                    }
                    None => {
                        wb.put(key.as_bytes(), value.as_bytes());
                    }
                }
                Ok(())
            } else {
                Err("WriteBatch not initialized".into())
            };
            debug!("Write batch put result: {:?}", result);
            result
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn write_batch_merge(
        &self,
        key: String,
        value: String,
        cf_name: Option<String>,
    ) -> Result<(), String> {
        debug!(
            "Write batch merge with key: {}, value: {}, cf_name: {:?}",
            key, value, cf_name
        );
        let db = self.db.read().unwrap();
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref db) = *db {
            let result = if let Some(ref mut wb) = *batch {
                match cf_name {
                    Some(cf_name) => {
                        let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                        wb.merge_cf(&cf, key.as_bytes(), value.as_bytes());
                    }
                    None => {
                        wb.merge(key.as_bytes(), value.as_bytes());
                    }
                }
                Ok(())
            } else {
                Err("WriteBatch not initialized".into())
            };
            debug!("Write batch merge result: {:?}", result);
            result
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn write_batch_delete(&self, key: String, cf_name: Option<String>) -> Result<(), String> {
        debug!(
            "Write batch delete with key: {}, cf_name: {:?}",
            key, cf_name
        );
        let db = self.db.read().unwrap();
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref db) = *db {
            let result = if let Some(ref mut wb) = *batch {
                match cf_name {
                    Some(cf_name) => {
                        let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                        wb.delete_cf(&cf, key.as_bytes());
                    }
                    None => {
                        wb.delete(key.as_bytes());
                    }
                }
                Ok(())
            } else {
                Err("WriteBatch not initialized".into())
            };
            debug!("Write batch delete result: {:?}", result);
            result
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn write_batch_write(&self) -> Result<(), String> {
        debug!("Write batch write");
        let db = self.db.read().unwrap();
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref db) = *db {
            let result = if let Some(wb) = batch.take() {
                db.write(wb).map_err(|e| e.to_string())?;
                *batch = Some(WriteBatchWithTransaction::default());
                Ok(())
            } else {
                Err("WriteBatch not initialized".into())
            };
            debug!("Write batch write result: {:?}", result);
            result
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn write_batch_clear(&self) -> Result<(), String> {
        debug!("Write batch clear");
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref mut wb) = *batch {
            wb.clear();
            Ok(())
        } else {
            Err("WriteBatch not initialized".into())
        }
    }

    pub fn write_batch_destroy(&self) -> Result<(), String> {
        debug!("Write batch destroy");
        let mut batch = self.write_batch.lock().unwrap();
        *batch = None;
        Ok(())
    }

    pub fn create_iterator(&self) -> usize {
        debug!("Creating iterator");
        let mut iterators = self.iterators.lock().unwrap();
        let id = self.iterator_id_counter.fetch_add(1, Ordering::SeqCst);
        iterators.insert(id, (vec![], rust_rocksdb::Direction::Forward));
        id
    }

    pub fn destroy_iterator(&self, iterator_id: usize) -> Result<(), String> {
        debug!("Destroying iterator with id: {}", iterator_id);
        let mut iterators = self.iterators.lock().unwrap();
        if iterators.remove(&iterator_id).is_some() {
            Ok(())
        } else {
            Err("Iterator ID not found".to_string())
        }
    }

    pub fn iterator_seek(
        &self,
        iterator_id: usize,
        key: String,
        direction: rust_rocksdb::Direction,
    ) -> Result<String, String> {
        match direction {
            rust_rocksdb::Direction::Forward => {
                debug!(
                    "Iterator seek with id: {}, key: {}, direction: Forward",
                    iterator_id, key
                );
            }
            rust_rocksdb::Direction::Reverse => {
                debug!(
                    "Iterator seek with id: {}, key: {}, direction: Reverse",
                    iterator_id, key
                );
            }
        }
        let db = self.db.read().unwrap();
        if let Some(ref db) = *db {
            let mut iterators = self.iterators.lock().unwrap();
            if let Some(iterator) = iterators.get_mut(&iterator_id) {
                let mut iter =
                    db.iterator(rust_rocksdb::IteratorMode::From(key.as_bytes(), direction));
                if let Some(Ok((k, v))) = iter.next() {
                    iterator.0 = k.to_vec();
                    iterator.1 = direction;
                    let result = Ok(format!(
                        "{}:{}",
                        String::from_utf8(k.to_vec()).unwrap(),
                        String::from_utf8(v.to_vec()).unwrap()
                    ));
                    debug!("Iterator seek result: {:?}", result);
                    result
                } else {
                    Ok(String::from("invalid:invalid"))
                }
            } else {
                Err("Iterator ID not found".to_string())
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn iterator_next(&self, iterator_id: usize) -> Result<String, String> {
        debug!("Iterator next with id: {}", iterator_id);
        let db = self.db.read().unwrap();
        let mut iterators = self.iterators.lock().unwrap();
        if let Some(ref db) = *db {
            if let Some(iterator) = iterators.get_mut(&iterator_id) {
                let (ref mut pos, direction) = *iterator;
                let mut iter = db.iterator(rust_rocksdb::IteratorMode::From(pos, direction));
                iter.next(); // Move to current position
                if let Some(Ok((k, v))) = iter.next() {
                    pos.clear();
                    pos.extend_from_slice(&k);
                    let result = Ok(format!(
                        "{}:{}",
                        String::from_utf8(k.to_vec()).unwrap(),
                        String::from_utf8(v.to_vec()).unwrap()
                    ));
                    debug!("Iterator next result: {:?}", result);
                    result
                } else {
                    Ok(String::from("invalid:invalid"))
                }
            } else {
                Err("Iterator ID not found".to_string())
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn iterator_prev(&self, iterator_id: usize) -> Result<String, String> {
        debug!("Iterator prev with id: {}", iterator_id);
        let db = self.db.read().unwrap();
        if let Some(ref db) = *db {
            let mut iterators = self.iterators.lock().unwrap();
            if let Some(iterator) = iterators.get_mut(&iterator_id) {
                let (ref mut pos, _direction) = *iterator;
                let mut iter = db.iterator(rust_rocksdb::IteratorMode::From(
                    pos,
                    rust_rocksdb::Direction::Reverse,
                ));
                iter.next(); // Move to current position
                if let Some(Ok((k, v))) = iter.next() {
                    pos.clear();
                    pos.extend_from_slice(&k);
                    let result = Ok(format!(
                        "{}:{}",
                        String::from_utf8(k.to_vec()).unwrap(),
                        String::from_utf8(v.to_vec()).unwrap()
                    ));
                    debug!("Iterator prev result: {:?}", result);
                    result
                } else {
                    Ok(String::from("invalid:invalid"))
                }
            } else {
                Err("Iterator ID not found".to_string())
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn backup(&self) -> Result<(), String> {
        info!("Creating backup");
        let backup_path = format!("{}/backup", self.db_path);
        let backup_opts = BackupEngineOptions::new(&backup_path).map_err(|e| e.to_string())?;
        let mut backup_engine =
            BackupEngine::open(&backup_opts, &Env::new().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;

        let db = self.db.read().unwrap();
        if let Some(ref db) = *db {
            let result = backup_engine
                .create_new_backup(db)
                .map_err(|e| e.to_string());
            debug!("Backup result: {:?}", result);
            result
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn restore_latest_backup(&self) -> Result<(), String> {
        info!("Restoring latest backup");
        let backup_path = format!("{}/backup", self.db_path);
        let backup_opts = BackupEngineOptions::new(&backup_path).map_err(|e| e.to_string())?;
        let mut backup_engine =
            BackupEngine::open(&backup_opts, &Env::new().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;

        let restore_opts = RestoreOptions::default();
        let result = backup_engine
            .restore_from_latest_backup(
                Path::new(&self.db_path),
                Path::new(&self.db_path),
                &restore_opts,
            )
            .map_err(|e| e.to_string())?;
        self.reload()?;
        debug!("Restore latest backup result: {:?}", result);
        Ok(())
    }

    pub fn restore_backup(&self, backup_id: u32) -> Result<(), String> {
        info!("Restoring backup with id: {}", backup_id);
        let backup_path = format!("{}/backup", self.db_path);
        let backup_opts = BackupEngineOptions::new(&backup_path).map_err(|e| e.to_string())?;
        let mut backup_engine =
            BackupEngine::open(&backup_opts, &Env::new().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;

        let restore_opts = RestoreOptions::default();
        let result = backup_engine
            .restore_from_backup(
                Path::new(&self.db_path),
                Path::new(&self.db_path),
                &restore_opts,
                backup_id,
            )
            .map_err(|e| e.to_string())?;
        self.reload()?;
        debug!("Restore backup result: {:?}", result);
        Ok(())
    }

    pub fn get_backup_info(&self) -> Result<Vec<BackupInfo>, String> {
        info!("Getting backup info");
        let backup_path = format!("{}/backup", self.db_path);
        let backup_opts = BackupEngineOptions::new(&backup_path).map_err(|e| e.to_string())?;
        let backup_engine =
            BackupEngine::open(&backup_opts, &Env::new().map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;

        let info = backup_engine.get_backup_info();
        let backup_info: Vec<BackupInfo> = info.into_iter().map(BackupInfo::from).collect();
        debug!("Get backup info result: {:?}", backup_info);
        Ok(backup_info)
    }
}