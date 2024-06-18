use async_std::fs::OpenOptions;
use async_std::io;
use async_std::path::PathBuf;
use async_std::sync::{Arc, Mutex};
use async_std::task;
use log::LevelFilter;
use std::str::FromStr;
use std::fs;

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl FromStr for LogLevel {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err("no match"),
        }
    }
}

impl Into<LevelFilter> for LogLevel {
    fn into(self) -> LevelFilter {
        match self {
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}

impl LogLevel {
    pub fn variants() -> [&'static str; 4] {
        ["debug", "info", "warn", "error"]
    }
}

pub struct LockFileGuard {
    path: PathBuf,
    _file: Arc<Mutex<async_std::fs::File>>,
}

impl LockFileGuard {
    pub(crate) async fn new(path: PathBuf) -> io::Result<Self> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)
            .await?;

        let file = Arc::new(Mutex::new(file));

        // Implement a simple file lock mechanism by holding the file open
        {
            let _locked_file = file.lock().await;
        }

        Ok(Self { path, _file: file })
    }
}

impl Drop for LockFileGuard {
    fn drop(&mut self) {
        let path = self.path.clone();
        task::block_on(async {
            let _locked_file = self._file.lock().await;
            // _locked_file will be dropped here
            if let Err(e) = fs::remove_file(&path) {
                eprintln!("Failed to remove lock file: {}", e);
            }
        });
    }
}

// Helper function to create lock guard
pub async fn create_lock_guard(lock_file_path: PathBuf) -> Option<LockFileGuard> {
    LockFileGuard::new(lock_file_path).await.ok()
}
