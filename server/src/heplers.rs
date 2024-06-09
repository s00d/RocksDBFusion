use fs2::FileExt;
use log::LevelFilter;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::io;

#[derive(Debug)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(format!("Invalid log level: {}", s)),
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

pub struct LockFileGuard {
    path: PathBuf,
    _file: Arc<File>,
}

impl LockFileGuard {
    pub(crate) fn new(path: PathBuf, file: File) -> io::Result<Self> {
        file.lock_exclusive()?;
        Ok(Self {
            path,
            _file: Arc::new(file),
        })
    }
}

impl Drop for LockFileGuard {
    fn drop(&mut self) {
        if let Err(e) = self._file.unlock() {
            eprintln!("Failed to unlock file: {}", e);
        }
        if let Err(e) = std::fs::remove_file(&self.path) {
            eprintln!("Failed to remove lock file: {}", e);
        }
    }
}
