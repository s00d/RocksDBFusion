#[cfg(not(target_os = "windows"))]
use file_lock::{FileLock, FileOptions};
use log::LevelFilter;
#[cfg(not(target_os = "windows"))]
use std::path::PathBuf;
use std::str::FromStr;
#[cfg(not(target_os = "windows"))]
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

#[cfg(not(target_os = "windows"))]
pub struct LockFileGuard {
    path: PathBuf,
    _file: FileLock,
}
#[cfg(not(target_os = "windows"))]
impl LockFileGuard {
    pub(crate) fn new(path: PathBuf) -> io::Result<Self> {
        let options = FileOptions::new().write(true).create(true).append(false);
        let lock = FileLock::lock(path.to_str().unwrap(), true, options)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        Ok(Self {
            path,
            _file: lock,
        })
    }
}
#[cfg(not(target_os = "windows"))]
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

