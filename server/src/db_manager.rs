use std::sync::{Arc, Mutex};
use std::time::Duration;
use json_patch::{Patch, PatchOperation};
use rust_rocksdb::{DBWithThreadMode, SingleThreaded, Options, ColumnFamilyDescriptor, MergeOperands, WriteBatchWithTransaction};
use serde_json::{Value};

pub type DbInstance = Arc<Mutex<Option<DBWithThreadMode<SingleThreaded>>>>;

pub fn json_merge(
    _new_key: &[u8],
    existing_val: Option<&[u8]>,
    operands: &MergeOperands,
) -> Option<Vec<u8>> {
    // Decode the existing value
    let mut doc: Value = if let Some(val) = existing_val {
        serde_json::from_slice(val).unwrap_or(Value::Array(vec![]))
    } else {
        Value::Array(vec![])
    };

    // Process each operand
    for op in operands {
        if let Ok(patch) = serde_json::from_slice::<Vec<PatchOperation>>(op) {
            let patch = Patch(patch);
            if let Err(e) = json_patch::patch(&mut doc, &patch) {
                eprintln!("Failed to apply patch: {:?}", e);
            }
        } else {
            eprintln!("Failed to deserialize operand");
        }
    }

    // Serialize the updated JSON object back to bytes
    match serde_json::to_vec(&doc) {
        Ok(bytes) => Some(bytes),
        Err(e) => {
            eprintln!("Failed to serialize JSON: {:?}", e);
            None
        }
    }
}

pub struct RocksDBManager {
    pub db: DbInstance,
    pub db_path: String,
    write_batch: Mutex<Option<WriteBatchWithTransaction<false>>>,
}

impl RocksDBManager {
    pub fn new(db_path: &str, ttl_secs: Option<u64>) -> Result<Self, String> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_merge_operator_associative("json_merge", json_merge);

        let cf_names = DBWithThreadMode::<SingleThreaded>::list_cf(&opts, db_path).unwrap_or(vec!["default".to_string()]);
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
                DBWithThreadMode::<SingleThreaded>::open_cf_descriptors_with_ttl(&opts, db_path, cf_descriptors, duration).map_err(|e| e.to_string())?
            },
            None => {
                DBWithThreadMode::<SingleThreaded>::open_cf_descriptors(&opts, db_path, cf_descriptors).map_err(|e| e.to_string())?
            }
        };

        let db = Arc::new(Mutex::new(Some(db)));

        Ok(RocksDBManager {
            db,
            db_path: db_path.to_string(),
            write_batch: Mutex::new(Some(WriteBatchWithTransaction::default())),
        })
    }

    pub fn put(&self, key: String, value: String, cf_name: Option<String>) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        if let Some(ref db) = *db {
            match cf_name {
                Some(cf_name) => {
                    let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    db.put_cf(&cf, key.as_bytes(), value.as_bytes()).map_err(|e| e.to_string())
                }
                None => db.put(key.as_bytes(), value.as_bytes()).map_err(|e| e.to_string()),
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn get(&self, key: String, cf_name: Option<String>, default: Option<String>) -> Result<Option<String>, String> {
        let db = self.db.lock().unwrap();
        if let Some(ref db) = *db {
            match cf_name {
                Some(cf_name) => {
                    let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    match db.get_cf(&cf, key.as_bytes()) {
                        Ok(Some(value)) => Ok(Some(String::from_utf8(value).map_err(|e| e.to_string())?)),
                        Ok(None) => Ok(default), // Если значение не найдено, возвращаем значение по умолчанию
                        Err(e) => Err(e.to_string()),
                    }
                }
                None => {
                    match db.get(key.as_bytes()) {
                        Ok(Some(value)) => Ok(Some(String::from_utf8(value).map_err(|e| e.to_string())?)),
                        Ok(None) => Ok(default), // Если значение не найдено, возвращаем значение по умолчанию
                        Err(e) => Err(e.to_string()),
                    }
                }
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn delete(&self, key: String, cf_name: Option<String>) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        if let Some(ref db) = *db {
            match cf_name {
                Some(cf_name) => {
                    let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    db.delete_cf(&cf, key.as_bytes()).map_err(|e| e.to_string())
                }
                None => db.delete(key.as_bytes()).map_err(|e| e.to_string()),
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn merge(&self, key: String, value: String, cf_name: Option<String>) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        if let Some(ref db) = *db {
            match cf_name {
                Some(cf_name) => {
                    let cf = db.cf_handle(&cf_name).ok_or("Column family not found")?;
                    db.merge_cf(&cf, key.as_bytes(), value.as_bytes()).map_err(|e| e.to_string())
                }
                None => db.merge(key.as_bytes(), value.as_bytes()).map_err(|e| e.to_string()),
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn get_keys(&self, start: usize, limit: usize, query: Option<String>) -> Result<Vec<String>, String> {
        let db = self.db.lock().unwrap();
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
                .skip(start)
                .take(limit)
                .collect();

            Ok(keys)
        } else {
            Err("Database is not open".to_string())
        }
    }


    pub fn close(&self) -> Result<(), String> {
        let mut db_lock = self.db.lock().unwrap();
        *db_lock = None;
        Ok(())
    }

    pub fn reopen(&self) -> Result<(), String> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_merge_operator_associative("json_merge", json_merge);

        let cf_names = DBWithThreadMode::<SingleThreaded>::list_cf(&opts, &self.db_path).unwrap_or(vec!["default".to_string()]);
        let cf_descriptors: Vec<ColumnFamilyDescriptor> = cf_names
            .iter()
            .map(|name| {
                let mut cf_opts = Options::default();
                cf_opts.set_merge_operator_associative("json_merge", json_merge);
                ColumnFamilyDescriptor::new(name, cf_opts)
            })
            .collect();

        let new_db = DBWithThreadMode::<SingleThreaded>::open_cf_descriptors(&opts, &self.db_path, cf_descriptors)
            .map_err(|e| e.to_string())?;
        let mut db_lock = self.db.lock().unwrap();
        *db_lock = Some(new_db);
        Ok(())
    }

    pub fn reload(&self) -> Result<(), String> {
        self.close().unwrap();
        self.reopen().unwrap();

        println!("Database reloaded successfully.");
        Ok(())
    }

    pub fn list_column_families(&self, path: String) -> Result<Vec<String>, String> {
        let opts = Options::default();
        DBWithThreadMode::<SingleThreaded>::list_cf(&opts, path).map_err(|e| e.to_string())
    }

    pub fn create_column_family(&self, cf_name: String) -> Result<(), String> {
        let mut db = self.db.lock().unwrap();
        if let Some(ref mut db) = *db {
            let cf_exists = db.cf_handle(&cf_name).is_some();
            if cf_exists {
                Ok(())
            } else {
                let mut opts = Options::default();
                opts.set_merge_operator_associative("json_merge", json_merge);
                db.create_cf(&cf_name, &opts).map_err(|e| e.to_string())
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn drop_column_family(&self, cf_name: String) -> Result<(), String> {
        let mut db = self.db.lock().unwrap();
        if let Some(ref mut db) = *db {
            let cf_exists = db.cf_handle(&cf_name).is_some();
            if !cf_exists {
                Ok(())
            } else {
                db.drop_cf(&cf_name).map_err(|e| e.to_string())
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn compact_range(&self, start: Option<String>, end: Option<String>, cf_name: Option<String>) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        if let Some(ref db) = *db {
            match cf_name {
                Some(cf_name) => {
                    let cf = db.cf_handle(&cf_name).unwrap();
                    db.compact_range_cf(&cf, start.as_deref(), end.as_deref());
                    Ok(())
                }
                None => {
                    db.compact_range(start.as_deref(), end.as_deref());
                    Ok(())
                }
            }
        } else {
            Err("Database is not open".to_string())
        }
    }


    pub fn write_batch_put(&self, key: String, value: String, cf_name: Option<String>) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref db) = *db {
            if let Some(ref mut wb) = *batch {
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
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn write_batch_merge(&self, key: String, value: String, cf_name: Option<String>) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref db) = *db {
            if let Some(ref mut wb) = *batch {
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
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn write_batch_delete(&self, key: String, cf_name: Option<String>) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref db) = *db {
            if let Some(ref mut wb) = *batch {
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
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn write_batch_write(&self) -> Result<(), String> {
        let db = self.db.lock().unwrap();
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref db) = *db {
            if let Some(wb) = batch.take() {
                db.write(wb)
                    .map_err(|e| e.to_string())?;
                *batch = Some(WriteBatchWithTransaction::default());
                Ok(())
            } else {
                Err("WriteBatch not initialized".into())
            }
        } else {
            Err("Database is not open".to_string())
        }
    }

    pub fn write_batch_clear(&self) -> Result<(), String> {
        let mut batch = self.write_batch.lock().unwrap();
        if let Some(ref mut wb) = *batch {
            wb.clear();
            Ok(())
        } else {
            Err("WriteBatch not initialized".into())
        }
    }

    pub fn write_batch_destroy(&self) -> Result<(), String> {
        let mut batch = self.write_batch.lock().unwrap();
        *batch = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn setup_db(path: &str) -> RocksDBManager {
        if Path::new(path).exists() {
            std::fs::remove_dir_all(path).unwrap();
        }
        RocksDBManager::new(path, None).unwrap()
    }

    #[test]
    fn test_put_and_get() {
        let manager = setup_db(".temp/test_put_and_get");
        manager.put("key1".to_string(), "value1".to_string(), None).unwrap();
        let result = manager.get("key1".to_string(), None, None).unwrap();
        assert_eq!(result, Some("value1".to_string()));
    }

    #[test]
    fn test_delete() {
        let manager = setup_db(".temp/test_delete");
        manager.put("key1".to_string(), "value1".to_string(), None).unwrap();
        manager.delete("key1".to_string(), None).unwrap();
        let result = manager.get("key1".to_string(), None, None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_merge() {
        let manager = setup_db(".temp/test_merge");
        manager.put("key1".to_string(), r#"{"field": "value1"}"#.to_string(), None).unwrap();
        manager.merge("key1".to_string(), r#"{"field": "value2"}"#.to_string(), None).unwrap();
        let result = manager.get("key1".to_string(), None, None).unwrap();
        assert_eq!(result, Some(r#"{"field":"value2"}"#.to_string()));
    }

    #[test]
    fn test_list_column_families() {
        let manager = setup_db(".temp/test_list_column_families");
        manager.create_column_family("test_list_column_families".to_string()).unwrap();
        let result = manager.list_column_families(".temp/test_list_column_families".to_string()).unwrap();
        assert!(result.contains(&"test_list_column_families".to_string()));
    }

    #[test]
    fn test_create_and_drop_column_family() {
        let manager = setup_db(".temp/test_create_and_drop_column_family");
        manager.create_column_family("new_cf".to_string()).unwrap();
        manager.create_column_family("test_list_column_families".to_string()).unwrap();
        let result = manager.list_column_families(".temp/test_create_and_drop_column_family".to_string()).unwrap();
        assert!(result.contains(&"new_cf".to_string()));
        manager.drop_column_family("new_cf".to_string()).unwrap();
        let result = manager.list_column_families(".temp/test_create_and_drop_column_family".to_string()).unwrap();
        assert!(!result.contains(&"new_cf".to_string()));
    }

    #[test]
    fn test_compact_range() {
        let manager = setup_db(".temp/test_compact_range");
        manager.put("key1".to_string(), "value1".to_string(), None).unwrap();
        manager.put("key2".to_string(), "value2".to_string(), None).unwrap();
        manager.compact_range(Some("key1".to_string()), Some("key2".to_string()), None).unwrap();
    }

    #[test]
    fn test_write_batch_operations() {
        let manager = setup_db(".temp/test_write_batch_operations");
        manager.write_batch_put("key1".to_string(), "value1".to_string(), None).unwrap();
        manager.write_batch_merge("key1".to_string(), "value2".to_string(), None).unwrap();
        manager.write_batch_delete("key1".to_string(), None).unwrap();
        manager.write_batch_write().unwrap();
        let result = manager.get("key1".to_string(), None, None).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_write_batch_clear_and_destroy() {
        let manager = setup_db(".temp/test_write_batch_clear_and_destroy");
        manager.write_batch_put("key1".to_string(), "value1".to_string(), None).unwrap();
        manager.write_batch_clear().unwrap();
        manager.write_batch_write().unwrap();
        let result = manager.get("key1".to_string(), None, None).unwrap();
        assert_eq!(result, None);
        manager.write_batch_destroy().unwrap();
    }
}