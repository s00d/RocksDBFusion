use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use rust_rocksdb::{DB, Options, MergeOperands, ColumnFamilyDescriptor, DBWithThreadMode, SingleThreaded, TransactionDB, TransactionDBOptions, Transaction, TransactionOptions, WriteOptions, WriteBatchWithTransaction};
use rust_rocksdb::backup::{BackupEngine, BackupEngineInfo, BackupEngineOptions, RestoreOptions};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use bytes::BytesMut;
use tokio_stream::StreamExt;
use futures::SinkExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    action: String,
    key: Option<String>,
    value: Option<String>,
    cf_name: Option<String>,
    options: Option<HashMap<String, String>>,
    backup_path: Option<String>,
    num_backups_to_keep: Option<usize>,
    backup_id: Option<u32>,
    restore_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    success: bool,
    result: Option<String>,
    error: Option<String>,
}

type DbInstance = Arc<Mutex<DBWithThreadMode<SingleThreaded>>>;
type TxnDbInstance = Arc<Mutex<Option<RocksDBTransaction>>>;
type BackupEngineInstance = Arc<Mutex<Option<BackupEngine>>>;
type WriteBatchInstance = Arc<Mutex<Option<RocksDBWriteBatch>>>;

fn json_merge(
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
        if let Ok(patch) = serde_json::from_slice::<Value>(op) {
            let p: json_patch::Patch = serde_json::from_value(patch).unwrap();
            json_patch::patch(&mut doc, &p).unwrap();
        }
    }

    Some(serde_json::to_vec(&doc).unwrap())
}

pub struct RocksDBTransaction {
    transaction_db: Arc<TransactionDB<SingleThreaded>>,
    transaction: Arc<Mutex<Option<Transaction<'static, TransactionDB<SingleThreaded>>>>>,
}

fn create_transaction(transaction_db: &Arc<TransactionDB<SingleThreaded>>) -> Transaction<'static, TransactionDB<SingleThreaded>> {
    let txn_opts = TransactionOptions::default();
    let write_opts = WriteOptions::default();
    unsafe {
        std::mem::transmute::<Transaction<TransactionDB<SingleThreaded>>, Transaction<'static, TransactionDB<SingleThreaded>>>(
            transaction_db.transaction_opt(&write_opts, &txn_opts),
        )
    }
}

impl RocksDBTransaction {
    pub fn new(path: String) -> Result<Self, String> {
        let txn_db_opts = TransactionDBOptions::default();
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let transaction_db = TransactionDB::<SingleThreaded>::open(&opts, &txn_db_opts, &path)
            .map_err(|e| e.to_string())?;

        let transaction_db = Arc::new(transaction_db);
        let transaction = create_transaction(&transaction_db);

        Ok(RocksDBTransaction {
            transaction_db: Arc::clone(&transaction_db),
            transaction: Arc::new(Mutex::new(Some(transaction))),
        })
    }

    pub fn commit(&self) -> Result<(), String> {
        let mut txn_guard = self.transaction.lock().unwrap();
        if let Some(txn) = txn_guard.take() {
            txn.commit().map_err(|e| e.to_string())?;
            *txn_guard = Some(create_transaction(&self.transaction_db));
            Ok(())
        } else {
            Err("No active transaction".to_string())
        }
    }

    pub fn rollback(&self) -> Result<(), String> {
        let mut txn_guard = self.transaction.lock().unwrap();
        if let Some(txn) = txn_guard.take() {
            txn.rollback().map_err(|e| e.to_string())?;
            *txn_guard = Some(create_transaction(&self.transaction_db));
            Ok(())
        } else {
            Err("No active transaction".to_string())
        }
    }

    pub fn set_savepoint(&self) {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            txn.set_savepoint();
        }
    }

    pub fn rollback_to_savepoint(&self) -> Result<(), String> {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            txn.rollback_to_savepoint().map_err(|e| e.to_string())
        } else {
            Err("No active transaction".to_string())
        }
    }

    pub fn put(&self, key: String, value: String, cf_name: Option<String>) -> Result<(), String> {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            match cf_name {
                Some(cf_name) => {
                    let cf = self.transaction_db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    txn.put_cf(&cf, key.as_bytes(), value.as_bytes()).map_err(|e| e.to_string())
                }
                None => txn.put(key.as_bytes(), value.as_bytes()).map_err(|e| e.to_string()),
            }
        } else {
            Err("No active transaction".to_string())
        }
    }

    pub fn get(&self, key: String, cf_name: Option<String>) -> Result<Option<String>, String> {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            match cf_name {
                Some(cf_name) => {
                    let cf = self.transaction_db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    match txn.get_cf(&cf, key.as_bytes()) {
                        Ok(Some(value)) => Ok(Some(String::from_utf8(value).map_err(|e| e.to_string())?)),
                        Ok(None) => Ok(None),
                        Err(e) => Err(e.to_string()),
                    }
                }
                None => {
                    match txn.get(key.as_bytes()) {
                        Ok(Some(value)) => Ok(Some(String::from_utf8(value).map_err(|e| e.to_string())?)),
                        Ok(None) => Ok(None),
                        Err(e) => Err(e.to_string()),
                    }
                }
            }
        } else {
            Err("No active transaction".to_string())
        }
    }

    pub fn delete(&self, key: String, cf_name: Option<String>) -> Result<(), String> {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            match cf_name {
                Some(cf_name) => {
                    let cf = self.transaction_db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    txn.delete_cf(&cf, key.as_bytes()).map_err(|e| e.to_string())
                }
                None => txn.delete(key.as_bytes()).map_err(|e| e.to_string()),
            }
        } else {
            Err("No active transaction".to_string())
        }
    }

    pub fn merge(&self, key: String, value: String, cf_name: Option<String>) -> Result<(), String> {
        let txn_guard = self.transaction.lock().unwrap();
        if let Some(ref txn) = *txn_guard {
            match cf_name {
                Some(cf_name) => {
                    let cf = self.transaction_db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    txn.merge_cf(&cf, key.as_bytes(), value.as_bytes()).map_err(|e| e.to_string())
                }
                None => txn.merge(key.as_bytes(), value.as_bytes()).map_err(|e| e.to_string()),
            }
        } else {
            Err("No active transaction".to_string())
        }
    }
}

pub struct RocksDBWriteBatch {
    db: Arc<DB>,
    write_batch: Mutex<Option<WriteBatchWithTransaction<false>>>,
}

impl RocksDBWriteBatch {
    pub fn new(path: String, ttl_secs: Option<u64>) -> Result<Self, String> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_open_files(1000);
        opts.set_log_level(rust_rocksdb::LogLevel::Warn);

        let db = match ttl_secs {
            Some(ttl) => {
                let duration = Duration::from_secs(ttl);
                DB::open_with_ttl(&opts, &path, duration)
            }
            None => DB::open(&opts, &path),
        };

        match db {
            Ok(db) => Ok(RocksDBWriteBatch {
                db: Arc::new(db),
                write_batch: Mutex::new(None),
            }),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn start(&self) -> Result<(), String> {
        let mut batch = self.write_batch.lock().unwrap();
        *batch = Some(WriteBatchWithTransaction::<false>::default());
        Ok(())
    }

    pub fn put(&self, key: String, value: String, cf_name: Option<String>) -> Result<(), String> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref mut wb) = *batch {
            match cf_name {
                Some(cf_name) => {
                    let cf = self
                        .db
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    wb.put_cf(&cf, key.as_bytes(), value.as_bytes());
                }
                None => {
                    wb.put(key.as_bytes(), value.as_bytes());
                }
            }
        } else {
            return Err("WriteBatch not initialized".into());
        }
        Ok(())
    }

    pub fn merge(&self, key: String, value: String, cf_name: Option<String>) -> Result<(), String> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref mut wb) = *batch {
            match cf_name {
                Some(cf_name) => {
                    let cf = self
                        .db
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    wb.merge_cf(&cf, key.as_bytes(), value.as_bytes());
                }
                None => {
                    wb.merge(key.as_bytes(), value.as_bytes());
                }
            }
        } else {
            return Err("WriteBatch not initialized".into());
        }
        Ok(())
    }

    pub fn delete(&self, key: String, cf_name: Option<String>) -> Result<(), String> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref mut wb) = *batch {
            match cf_name {
                Some(cf_name) => {
                    let cf = self
                        .db
                        .cf_handle(&cf_name)
                        .ok_or("Column family not found")?;
                    wb.delete_cf(&cf, key.as_bytes());
                }
                None => {
                    wb.delete(key.as_bytes());
                }
            }
        } else {
            return Err("WriteBatch not initialized".into());
        }
        Ok(())
    }

    pub fn write(&self) -> Result<(), String> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(wb) = batch.take() {
            self.db
                .write(wb)
                .map_err(|e| e.to_string())?;
        } else {
            return Err("WriteBatch not initialized".into());
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<(), String> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref mut wb) = *batch {
            wb.clear();
        } else {
            return Err("WriteBatch not initialized".into());
        }
        Ok(())
    }

    pub fn destroy(&self) -> Result<(), String> {
        let mut batch = self.write_batch.lock().unwrap();
        *batch = None;
        Ok(())
    }
}

async fn handle_client(
    db: DbInstance,
    txn_db: TxnDbInstance,
    backup_engine: BackupEngineInstance,
    mut write_batch: WriteBatchInstance,
    mut socket: Framed<TcpStream, LengthDelimitedCodec>
) {
    while let Some(Ok(frame)) = socket.next().await {
        let req: Request = serde_json::from_slice(&frame).unwrap();
        let response = match req.action.as_str() {
            "put" => handle_put(&db, req.key, req.value, req.cf_name).await,
            "get" => handle_get(&db, req.key, req.cf_name).await,
            "delete" => handle_delete(&db, req.key, req.cf_name).await,
            "merge" => handle_merge(&db, req.key, req.value, req.cf_name).await,
            "list_column_families" => handle_list_column_families(req.value).await,
            "create_column_family" => handle_create_column_family(&db, req.value).await,
            "drop_column_family" => handle_drop_column_family(&db, req.value).await,
            "compact_range" => handle_compact_range(&db, req.key, req.value, req.cf_name).await,
            "begin_transaction" => handle_begin_transaction(&txn_db).await,
            "commit_transaction" => handle_commit_transaction(&txn_db).await,
            "rollback_transaction" => handle_rollback_transaction(&txn_db).await,
            "set_savepoint" => handle_set_savepoint(&txn_db).await,
            "rollback_to_savepoint" => handle_rollback_to_savepoint(&txn_db).await,
            "backup_create" => handle_backup_create(&backup_engine, &db).await,
            "backup_info" => handle_backup_info(&backup_engine).await,
            "backup_purge_old" => handle_backup_purge_old(&backup_engine, req.num_backups_to_keep).await,
            "backup_restore" => handle_backup_restore(&backup_engine, req.backup_id, req.restore_path).await,
            "write_batch_start" => handle_write_batch_start(&mut write_batch).await,
            "write_batch_put" => handle_write_batch_put(&mut write_batch, req.key, req.value, req.cf_name).await,
            "write_batch_merge" => handle_write_batch_merge(&mut write_batch, req.key, req.value, req.cf_name).await,
            "write_batch_delete" => handle_write_batch_delete(&mut write_batch, req.key, req.cf_name).await,
            "write_batch_write" => handle_write_batch_write(&mut write_batch).await,
            "write_batch_clear" => handle_write_batch_clear(&mut write_batch).await,
            "write_batch_destroy" => handle_write_batch_destroy(&mut write_batch).await,
            "seek_to_first" => handle_seek_to_first(&db).await,
            "seek_to_last" => handle_seek_to_last(&db).await,
            "seek" => handle_seek(&db, req.key.unwrap_or_default()).await,
            "seek_for_prev" => handle_seek_for_prev(&db, req.key.unwrap_or_default()).await,
            "valid" => handle_valid(&db).await,
            "next" => handle_next(&db).await,
            "prev" => handle_prev(&db).await,
            _ => Response { success: false, result: None, error: Some("Unknown action".to_string()) },
        };
        let response_bytes = serde_json::to_vec(&response).unwrap();
        socket.send(BytesMut::from(&response_bytes[..]).into()).await.unwrap();
    }
}

async fn handle_put(
    db: &DbInstance,
    key: Option<String>,
    value: Option<String>,
    cf_name: Option<String>
) -> Response {
    match (key, value) {
        (Some(key), Some(value)) => {
            let result = match cf_name {
                Some(cf_name) => {
                    let db = db.lock().unwrap();
                    let cf = db.cf_handle(&cf_name).unwrap();
                    db.put_cf(&cf, key, value)
                }
                None => db.lock().unwrap().put(key, value),
            };
            match result {
                Ok(_) => Response { success: true, result: None, error: None },
                Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
            }
        }
        _ => Response { success: false, result: None, error: Some("Missing key or value".to_string()) }
    }
}

async fn handle_get(
    db: &DbInstance,
    key: Option<String>,
    cf_name: Option<String>
) -> Response {
    match key {
        Some(key) => {
            let result = match cf_name {
                Some(cf_name) => {
                    let db = db.lock().unwrap();
                    let cf = db.cf_handle(&cf_name).unwrap();
                    db.get_cf(&cf, key)
                }
                None => db.lock().unwrap().get(key),
            };
            match result {
                Ok(Some(value)) => Response { success: true, result: Some(String::from_utf8(value).unwrap()), error: None },
                Ok(None) => Response { success: false, result: None, error: Some("Key not found".to_string()) },
                Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
            }
        }
        None => Response { success: false, result: None, error: Some("Missing key".to_string()) }
    }
}

async fn handle_delete(
    db: &DbInstance,
    key: Option<String>,
    cf_name: Option<String>
) -> Response {
    match key {
        Some(key) => {
            let result = match cf_name {
                Some(cf_name) => {
                    let db = db.lock().unwrap();
                    let cf = db.cf_handle(&cf_name).unwrap();
                    db.delete_cf(&cf, key)
                }
                None => db.lock().unwrap().delete(key),
            };
            match result {
                Ok(_) => Response { success: true, result: None, error: None },
                Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
            }
        }
        None => Response { success: false, result: None, error: Some("Missing key".to_string()) }
    }
}

async fn handle_merge(
    db: &DbInstance,
    key: Option<String>,
    value: Option<String>,
    cf_name: Option<String>
) -> Response {
    match (key, value) {
        (Some(key), Some(value)) => {
            let result = match cf_name {
                Some(cf_name) => {
                    let db = db.lock().unwrap();
                    let cf = db.cf_handle(&cf_name).unwrap();
                    db.merge_cf(&cf, key, value)
                }
                None => db.lock().unwrap().merge(key, value),
            };
            match result {
                Ok(_) => Response { success: true, result: None, error: None },
                Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
            }
        }
        _ => Response { success: false, result: None, error: Some("Missing key or value".to_string()) }
    }
}

async fn handle_list_column_families(path: Option<String>) -> Response {
    match path {
        Some(path) => {
            let opts = Options::default();
            match DB::list_cf(&opts, &path) {
                Ok(cfs) => Response { success: true, result: Some(serde_json::to_string(&cfs).unwrap()), error: None },
                Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
            }
        }
        None => Response { success: false, result: None, error: Some("Missing path".to_string()) }
    }
}

async fn handle_create_column_family(
    db: &DbInstance,
    cf_name: Option<String>
) -> Response {
    match cf_name {
        Some(cf_name) => {
            let cf_exists = db.lock().unwrap().cf_handle(&cf_name).is_some();
            if cf_exists {
                Response { success: true, result: None, error: None }
            } else {
                let mut opts = Options::default();
                opts.set_merge_operator_associative("json_merge", json_merge);
                match db.lock().unwrap().create_cf(&cf_name, &opts) {
                    Ok(_) => Response { success: true, result: None, error: None },
                    Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
                }
            }
        }
        None => Response { success: false, result: None, error: Some("Missing column family name".to_string()) }
    }
}

async fn handle_drop_column_family(
    db: &DbInstance,
    cf_name: Option<String>
) -> Response {
    match cf_name {
        Some(cf_name) => {
            let cf_exists = db.lock().unwrap().cf_handle(&cf_name).is_some();
            if !cf_exists {
                Response { success: true, result: None, error: None }
            } else {
                match db.lock().unwrap().drop_cf(&cf_name) {
                    Ok(_) => Response { success: true, result: None, error: None },
                    Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
                }
            }
        }
        None => Response { success: false, result: None, error: Some("Missing column family name".to_string()) }
    }
}

async fn handle_compact_range(
    db: &DbInstance,
    start: Option<String>,
    end: Option<String>,
    cf_name: Option<String>
) -> Response {
    match cf_name {
        Some(cf_name) => {
            let db = db.lock().unwrap();
            let cf = db.cf_handle(&cf_name).unwrap();
            db.compact_range_cf(&cf, start.as_deref(), end.as_deref());
        }
        None => {
            db.lock().unwrap().compact_range(start.as_deref(), end.as_deref());
        }
    }
    Response { success: true, result: None, error: None }
}

async fn handle_begin_transaction(txn_db: &TxnDbInstance) -> Response {
    let mut txn_db = txn_db.lock().unwrap();
    if txn_db.is_none() {
        *txn_db = Some(RocksDBTransaction::new("path_to_your_db".to_string()).unwrap());
        Response { success: true, result: None, error: None }
    } else {
        Response { success: false, result: None, error: Some("Transaction already in progress".to_string()) }
    }
}

async fn handle_commit_transaction(txn_db: &TxnDbInstance) -> Response {
    let mut txn_db = txn_db.lock().unwrap();
    if let Some(ref txn) = *txn_db {
        match txn.commit() {
            Ok(_) => {
                *txn_db = None;
                Response { success: true, result: None, error: None }
            },
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    } else {
        Response { success: false, result: None, error: Some("No active transaction".to_string()) }
    }
}

async fn handle_rollback_transaction(txn_db: &TxnDbInstance) -> Response {
    let mut txn_db = txn_db.lock().unwrap();
    if let Some(ref txn) = *txn_db {
        match txn.rollback() {
            Ok(_) => {
                *txn_db = None;
                Response { success: true, result: None, error: None }
            },
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    } else {
        Response { success: false, result: None, error: Some("No active transaction".to_string()) }
    }
}

async fn handle_set_savepoint(txn_db: &TxnDbInstance) -> Response {
    let txn_db = txn_db.lock().unwrap();
    if let Some(ref txn) = *txn_db {
        txn.set_savepoint();
        Response { success: true, result: None, error: None }
    } else {
        Response { success: false, result: None, error: Some("No active transaction".to_string()) }
    }
}

async fn handle_rollback_to_savepoint(txn_db: &TxnDbInstance) -> Response {
    let txn_db = txn_db.lock().unwrap();
    if let Some(ref txn) = *txn_db {
        match txn.rollback_to_savepoint() {
            Ok(_) => Response { success: true, result: None, error: None },
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    } else {
        Response { success: false, result: None, error: Some("No active transaction".to_string()) }
    }
}

async fn handle_backup_create(backup_engine: &BackupEngineInstance, db: &DbInstance) -> Response {
    let mut backup_engine = backup_engine.lock().unwrap();
    let db = db.lock().unwrap();
    if let Some(be) = backup_engine.as_mut() {
        match be.create_new_backup(&*db) {
            Ok(_) => Response { success: true, result: None, error: None },
            Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
        }
    } else {
        Response { success: false, result: None, error: Some("Backup engine not initialized".to_string()) }
    }
}

#[derive(Debug, Serialize)]
struct BackupInfo {
    id: u32,
    size: u64,
    num_files: u32,
    timestamp: i64,
}

impl From<&BackupEngineInfo> for BackupInfo {
    fn from(info: &BackupEngineInfo) -> Self {
        BackupInfo {
            id: info.backup_id,
            size: info.size,
            num_files: info.num_files,
            timestamp: info.timestamp,
        }
    }
}


async fn handle_backup_info(backup_engine: &BackupEngineInstance) -> Response {
    let backup_engine = backup_engine.lock().unwrap();
    if let Some(be) = backup_engine.as_ref() {
        let backup_info = be.get_backup_info();
        let info: Vec<BackupInfo> = backup_info.iter().map(BackupInfo::from).collect();
        let result = serde_json::to_string(&info).unwrap();
        Response { success: true, result: Some(result), error: None }
    } else {
        Response { success: false, result: None, error: Some("Backup engine not initialized".to_string()) }
    }
}

async fn handle_backup_purge_old(backup_engine: &BackupEngineInstance, num_backups_to_keep: Option<usize>) -> Response {
    let mut backup_engine = backup_engine.lock().unwrap();
    if let Some(be) = backup_engine.as_mut() {
        match be.purge_old_backups(num_backups_to_keep.unwrap_or(0)) {
            Ok(_) => Response { success: true, result: None, error: None },
            Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
        }
    } else {
        Response { success: false, result: None, error: Some("Backup engine not initialized".to_string()) }
    }
}

async fn handle_backup_restore(backup_engine: &BackupEngineInstance, backup_id: Option<u32>, restore_path: Option<String>) -> Response {
    let mut backup_engine = backup_engine.lock().unwrap();
    if let Some(be) = backup_engine.as_mut() {
        match (backup_id, restore_path) {
            (Some(backup_id), Some(restore_path)) => {
                let opts = RestoreOptions::default();
                match be.restore_from_backup(&restore_path, &restore_path, &opts, backup_id) {
                    Ok(_) => Response { success: true, result: None, error: None },
                    Err(e) => Response { success: false, result: None, error: Some(e.to_string()) },
                }
            }
            _ => Response { success: false, result: None, error: Some("Missing backup ID or restore path".to_string()) }
        }
    } else {
        Response { success: false, result: None, error: Some("Backup engine not initialized".to_string()) }
    }
}

async fn handle_write_batch_start(write_batch: &mut WriteBatchInstance) -> Response {
    let mut batch = write_batch.lock().unwrap();
    *batch = Some(RocksDBWriteBatch::new("path_to_your_db".to_string(), None).unwrap());
    Response { success: true, result: None, error: None }
}

async fn handle_write_batch_put(write_batch: &mut WriteBatchInstance, key: Option<String>, value: Option<String>, cf_name: Option<String>) -> Response {
    match (key, value) {
        (Some(key), Some(value)) => {
            let batch = write_batch.lock().unwrap();
            if let Some(ref wb) = *batch {
                match wb.put(key, value, cf_name) {
                    Ok(_) => Response { success: true, result: None, error: None },
                    Err(e) => Response { success: false, result: None, error: Some(e) },
                }
            } else {
                Response { success: false, result: None, error: Some("WriteBatch not initialized".to_string()) }
            }
        }
        _ => Response { success: false, result: None, error: Some("Missing key or value".to_string()) }
    }
}

async fn handle_write_batch_merge(write_batch: &mut WriteBatchInstance, key: Option<String>, value: Option<String>, cf_name: Option<String>) -> Response {
    match (key, value) {
        (Some(key), Some(value)) => {
            let batch = write_batch.lock().unwrap();
            if let Some(ref wb) = *batch {
                match wb.merge(key, value, cf_name) {
                    Ok(_) => Response { success: true, result: None, error: None },
                    Err(e) => Response { success: false, result: None, error: Some(e) },
                }
            } else {
                Response { success: false, result: None, error: Some("WriteBatch not initialized".to_string()) }
            }
        }
        _ => Response { success: false, result: None, error: Some("Missing key or value".to_string()) }
    }
}

async fn handle_write_batch_delete(write_batch: &mut WriteBatchInstance, key: Option<String>, cf_name: Option<String>) -> Response {
    match key {
        Some(key) => {
            let batch = write_batch.lock().unwrap();
            if let Some(ref wb) = *batch {
                match wb.delete(key, cf_name) {
                    Ok(_) => Response { success: true, result: None, error: None },
                    Err(e) => Response { success: false, result: None, error: Some(e) },
                }
            } else {
                Response { success: false, result: None, error: Some("WriteBatch not initialized".to_string()) }
            }
        }
        None => Response { success: false, result: None, error: Some("Missing key".to_string()) }
    }
}

async fn handle_write_batch_write(write_batch: &mut WriteBatchInstance) -> Response {
    let batch = write_batch.lock().unwrap();
    if let Some(ref wb) = *batch {
        match wb.write() {
            Ok(_) => Response { success: true, result: None, error: None },
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    } else {
        Response { success: false, result: None, error: Some("WriteBatch not initialized".to_string()) }
    }
}

async fn handle_write_batch_clear(write_batch: &mut WriteBatchInstance) -> Response {
    let batch = write_batch.lock().unwrap();
    if let Some(ref wb) = *batch {
        match wb.clear() {
            Ok(_) => Response { success: true, result: None, error: None },
            Err(e) => Response { success: false, result: None, error: Some(e) },
        }
    } else {
        Response { success: false, result: None, error: Some("WriteBatch not initialized".to_string()) }
    }
}

async fn handle_write_batch_destroy(write_batch: &mut WriteBatchInstance) -> Response {
    let batch = write_batch.lock().unwrap();
    if let Some(ref wb) = *batch {
        let _ = wb.destroy();
        Response { success: true, result: None, error: None }
    } else {
        Response { success: false, result: None, error: Some("WriteBatch not initialized".to_string()) }
    }
}

async fn handle_seek_to_first(db: &DbInstance) -> Response {
    let db = db.lock().unwrap();
    let mut iter = db.iterator(rust_rocksdb::IteratorMode::Start);
    if let Some(Ok((key, _))) = iter.next() {
        Response { success: true, result: Some(String::from_utf8(key.to_vec()).unwrap()), error: None }
    } else {
        Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
    }
}

async fn handle_seek_to_last(db: &DbInstance) -> Response {
    let db = db.lock().unwrap();
    let mut iter = db.iterator(rust_rocksdb::IteratorMode::End);
    if let Some(Ok((key, _))) = iter.next() {
        Response { success: true, result: Some(String::from_utf8(key.to_vec()).unwrap()), error: None }
    } else {
        Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
    }
}

async fn handle_seek(db: &DbInstance, key: String) -> Response {
    let db = db.lock().unwrap();
    let mut iter = db.iterator(rust_rocksdb::IteratorMode::From(key.as_bytes(), rust_rocksdb::Direction::Forward));
    if let Some(Ok((key, _))) = iter.next() {
        Response { success: true, result: Some(String::from_utf8(key.to_vec()).unwrap()), error: None }
    } else {
        Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
    }
}

async fn handle_seek_for_prev(db: &DbInstance, key: String) -> Response {
    let db = db.lock().unwrap();
    let mut iter = db.iterator(rust_rocksdb::IteratorMode::From(key.as_bytes(), rust_rocksdb::Direction::Reverse));
    if let Some(Ok((key, _))) = iter.next() {
        Response { success: true, result: Some(String::from_utf8(key.to_vec()).unwrap()), error: None }
    } else {
        Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
    }
}

async fn handle_valid(db: &DbInstance) -> Response {
    let db = db.lock().unwrap();
    let mut iter = db.iterator(rust_rocksdb::IteratorMode::Start);
    let valid = iter.next().is_some();
    Response { success: true, result: Some(valid.to_string()), error: None }
}

async fn handle_next(db: &DbInstance) -> Response {
    let db = db.lock().unwrap();
    let mut iter = db.iterator(rust_rocksdb::IteratorMode::Start);
    iter.next();
    if let Some(Ok((key, value))) = iter.next() {
        Response { success: true, result: Some(format!("{}:{}", String::from_utf8(key.to_vec()).unwrap(), String::from_utf8(value.to_vec()).unwrap())), error: None }
    } else {
        Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
    }
}

async fn handle_prev(db: &DbInstance) -> Response {
    let db = db.lock().unwrap();
    let mut iter = db.iterator(rust_rocksdb::IteratorMode::End);
    if let Some(Ok((key, value))) = iter.next() {
        Response { success: true, result: Some(format!("{}:{}", String::from_utf8(key.to_vec()).unwrap(), String::from_utf8(value.to_vec()).unwrap())), error: None }
    } else {
        Response { success: false, result: None, error: Some("Iterator is invalid".to_string()) }
    }
}


#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <db_path> <port>", args[0]);
        return;
    }
    let path = &args[1];
    let port = &args[2];

    let addr = format!("127.0.0.1:{}", port);

    let mut opts = Options::default();
    opts.create_if_missing(true);
    opts.set_merge_operator_associative("json_merge", json_merge);

    let cf_names = DB::list_cf(&opts, path).unwrap_or(vec!["default".to_string()]);
    let cf_descriptors: Vec<ColumnFamilyDescriptor> = cf_names
        .iter()
        .map(|name| {
            let mut cf_opts = Options::default();
            cf_opts.set_merge_operator_associative("json_merge", json_merge);
            ColumnFamilyDescriptor::new(name, cf_opts)
        })
        .collect();

    let db = DBWithThreadMode::open_cf_descriptors(&opts, path, cf_descriptors).unwrap();
    let db = Arc::new(Mutex::new(db));

    let txn_db: Arc<Mutex<Option<RocksDBTransaction>>> = Arc::new(Mutex::new(None));

    let backup_engine_opts = BackupEngineOptions::new(path).unwrap();
    let env = rust_rocksdb::Env::new().unwrap();
    let backup_engine = BackupEngine::open(&backup_engine_opts, &env).unwrap();
    let backup_engine = Arc::new(Mutex::new(Some(backup_engine)));
    let write_batch: Arc<Mutex<Option<RocksDBWriteBatch>>> = Arc::new(Mutex::new(None));

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server listening on {}", addr);

    while let Ok((socket, _)) = listener.accept().await {
        let db = Arc::clone(&db);
        let txn_db = Arc::clone(&txn_db);
        let backup_engine = Arc::clone(&backup_engine);
        let write_batch = Arc::clone(&write_batch);
        let socket = Framed::new(socket, LengthDelimitedCodec::new());
        tokio::spawn(handle_client(db, txn_db, backup_engine, write_batch, socket));
    }
}
