pub mod db_manager;
mod helpers;
pub mod server;

use crate::helpers::{LogLevel};
#[cfg(not(target_os = "windows"))]
use crate::helpers::{LockFileGuard};
use env_logger::{Builder, Target};
use log::{info, LevelFilter};
use server::RocksDBServer;
use std::env;
use std::path::PathBuf;
use structopt::StructOpt;
use tokio::io;
use tokio::net::TcpListener;

#[derive(StructOpt, Debug)]
#[structopt(name = "RocksDB Server", about = "A simple RocksDB server.")]
struct Opt {
    /// Path to the RocksDB database
    #[structopt(long, short, default_value = "./db_test")]
    dbpath: PathBuf,

    /// Port to listen on
    #[structopt(long, short, default_value = "12345")]
    port: u16,

    /// Host to bind the server to
    #[structopt(long, default_value = "127.0.0.1")]
    host: String,

    /// Time-to-live (TTL) for database entries in seconds
    #[structopt(long, short)]
    ttl: Option<u64>,

    /// Authentication token for server access
    #[structopt(long)]
    token: Option<String>,

    /// Logging level (debug, info, warn, error)
    #[structopt(long, default_value = "info")]
    log_level: LogLevel,

    /// Path to the lock file
    #[structopt(long, parse(from_os_str))]
    lock_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let dbpath = if opt.dbpath.starts_with(".") {
        env::current_dir().unwrap().join(opt.dbpath)
    } else {
        opt.dbpath.clone()
    };
    let dbpath = dbpath.to_str().unwrap().to_string();

    let port = opt.port;
    let host = opt.host;
    let ttl = opt.ttl;
    let token = opt.token;

    #[cfg(not(target_os = "windows"))]
    let _lock_guard = if let Some(lock_file_path) = opt.lock_file {
        Some(LockFileGuard::new(lock_file_path)?)
    } else {
        None
    };

    #[cfg(target_os = "windows")]
    if opt.lock_file.is_some() {
        warn!("File locking is not supported on Windows. The lock file option will be ignored.");
    }

    let log_level: LevelFilter = opt.log_level.into();

    Builder::new()
        .filter(None, log_level)
        .target(Target::Stdout)
        .init();

    let addr = format!("{}:{}", host, port);

    let server = RocksDBServer::new(dbpath.clone(), ttl, token).map_err(|e| {
        log::error!("Failed to create RocksDBServer: {}", e);
        io::Error::new(io::ErrorKind::Other, "Failed to create RocksDBServer")
    })?;

    let listener = TcpListener::bind(&addr).await.map_err(|e| {
        log::error!("Failed to bind to address {}: {}", addr, e);
        io::Error::new(io::ErrorKind::AddrInUse, "Failed to bind to address")
    })?;
    info!("Server listening on {}", addr);

    tokio::spawn(async move {
        while let Ok((socket, _)) = listener.accept().await {
            let server = server.clone();
            tokio::spawn(async move {
                server.handle_client(socket).await;
            });
        }
    });

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received, terminating...");

    Ok(())
}
