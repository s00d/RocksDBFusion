pub mod db_manager;
mod helpers;
pub mod server;

#[cfg(not(target_os = "windows"))]
use crate::helpers::LockFileGuard;
use crate::helpers::LogLevel;
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
    #[structopt(long, short, env = "ROCKSDB_PATH", default_value = "./db_test", help = "Path to the RocksDB database")]
    dbpath: PathBuf,

    #[structopt(long, env = "ROCKSDB_HOST", default_value = "127.0.0.1", help = "Bind address")]
    host: String,

    #[structopt(long, short, env = "ROCKSDB_PORT", default_value = "12345", help = "Bind Port")]
    port: String,

    #[structopt(long, env = "ROCKSDB_TTL", short, help = "Time-to-live (TTL) for database entries in seconds")]
    ttl: Option<u64>,

    #[structopt(long, env = "ROCKSDB_TOKEN", help = "Authentication token for server access")]
    token: Option<String>,

    #[cfg(not(target_os = "windows"))]
    #[structopt(long, env = "ROCKSDB_LOCK_FILE", parse(from_os_str), help = "Path to the lock file")]
    lock_file: Option<PathBuf>,

    #[structopt(long, possible_values = &LogLevel::variants(), case_insensitive = true, env = "ROCKSDB_LOG_LEVEL", default_value = "info", help = "Logging level")]
    log_level: LogLevel,
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
