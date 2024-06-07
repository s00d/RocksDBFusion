use std::sync::{Arc, Mutex};
use rust_rocksdb::{backup::{BackupEngine, BackupEngineOptions, RestoreOptions, BackupEngineInfo}, DBWithThreadMode, SingleThreaded, Env};
use serde::{Deserialize, Serialize};

pub type BackupEngineInstance = Arc<Mutex<Option<BackupEngine>>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub(crate) id: u32,
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

#[derive(Clone)]
pub struct RocksDBBackupManager {
    pub backup_engine: BackupEngineInstance,
}

impl RocksDBBackupManager {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let backup_engine_opts = BackupEngineOptions::new(db_path).map_err(|e| e.to_string())?;
        let env = Env::new().map_err(|e| e.to_string())?;
        let backup_engine = BackupEngine::open(&backup_engine_opts, &env).map_err(|e| e.to_string())?;
        let backup_engine = Arc::new(Mutex::new(Some(backup_engine)));

        Ok(RocksDBBackupManager {
            backup_engine,
        })
    }

    pub fn create_backup(&self, db: &DBWithThreadMode<SingleThreaded>) -> Result<(), String> {
        let mut backup_engine = self.backup_engine.lock().unwrap();
        if let Some(be) = backup_engine.as_mut() {
            be.create_new_backup(db).map_err(|e| e.to_string())
        } else {
            Err("Backup engine not initialized".to_string())
        }
    }

    pub fn get_backup_info(&self) -> Result<Vec<BackupInfo>, String> {
        let backup_engine = self.backup_engine.lock().unwrap();
        if let Some(be) = backup_engine.as_ref() {
            let backup_info = be.get_backup_info();
            let info: Vec<BackupInfo> = backup_info.iter().map(BackupInfo::from).collect();
            Ok(info)
        } else {
            Err("Backup engine not initialized".to_string())
        }
    }

    pub fn purge_old_backups(&self, num_backups_to_keep: usize) -> Result<(), String> {
        let mut backup_engine = self.backup_engine.lock().unwrap();
        if let Some(be) = backup_engine.as_mut() {
            be.purge_old_backups(num_backups_to_keep).map_err(|e| e.to_string())
        } else {
            Err("Backup engine not initialized".to_string())
        }
    }

    pub fn restore_from_backup(&self, backup_id: u32, restore_path: String) -> Result<(), String> {
        let mut backup_engine = self.backup_engine.lock().unwrap();
        if let Some(be) = backup_engine.as_mut() {
            let opts = RestoreOptions::default();
            be.restore_from_backup(&restore_path, &restore_path, &opts, backup_id).map_err(|e| e.to_string())
        } else {
            Err("Backup engine not initialized".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::path::Path;
    use rust_rocksdb::{Options};

    fn setup_db(path: &str) -> Arc<DBWithThreadMode<SingleThreaded>> {
        if Path::new(path).exists() {
            std::fs::remove_dir_all(path).unwrap();
        }
        let mut opts = Options::default();
        opts.create_if_missing(true);
        Arc::new(DBWithThreadMode::<SingleThreaded>::open(&opts, path).unwrap())
    }

    #[test]
    fn test_create_backup() {
        let db = setup_db(".temp/test_create_backup");
        let manager = RocksDBBackupManager::new(".temp/test_create_backup").unwrap();
        manager.create_backup(&db).unwrap();
    }

    #[test]
    fn test_get_backup_info() {
        let db = setup_db(".temp/test_get_backup_info");
        let manager = RocksDBBackupManager::new(".temp/test_get_backup_info").unwrap();
        manager.create_backup(&db).unwrap();
        let backup_info = manager.get_backup_info().unwrap();
        assert!(backup_info.len() > 0);
    }

    #[test]
    fn test_purge_old_backups() {
        let db = setup_db(".temp/test_purge_old_backups");
        let manager = RocksDBBackupManager::new(".temp/test_purge_old_backups").unwrap();
        manager.create_backup(&db).unwrap();
        manager.purge_old_backups(0).unwrap();
        let backup_info = manager.get_backup_info().unwrap();
        assert_eq!(backup_info.len(), 0);
    }

    #[test]
    fn test_restore_from_backup() {
        let db = setup_db(".temp/test_restore_from_backup");
        let manager = RocksDBBackupManager::new(".temp/test_restore_from_backup").unwrap();
        manager.create_backup(&db).unwrap();
        manager.restore_from_backup(1, ".temp/test_restore_from_backup_restore".to_string()).unwrap();
    }
}
