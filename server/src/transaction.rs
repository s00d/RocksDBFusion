use std::sync::{Arc, Mutex};
use rust_rocksdb::{Options, SingleThreaded, Transaction, TransactionDB, TransactionDBOptions, TransactionOptions, WriteOptions};

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
