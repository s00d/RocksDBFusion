mod cache;
pub mod db_manager;
mod helpers;
mod queue;
pub mod server;

use async_std::channel::{bounded, Receiver};
use async_std::io::{prelude::*, BufReader, BufWriter};
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::Arc;
use async_std::task;
use futures::stream::StreamExt;
use futures::FutureExt;

use crate::helpers::{create_lock_guard, LogLevel};
use crate::server::{Request, RocksDBServer};
use log::{error, info, warn};
use std::env;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "RocksDB Server", about = "A simple RocksDB server.")]
struct Opt {
    #[structopt(
        long,
        short,
        env = "ROCKSDB_PATH",
        default_value = "./db_test",
        help = "Path to the RocksDB database"
    )]
    dbpath: PathBuf,

    #[structopt(
        long,
        env = "ROCKSDB_HOST",
        default_value = "127.0.0.1",
        help = "Bind address"
    )]
    host: String,

    #[structopt(
        long,
        short,
        env = "ROCKSDB_PORT",
        default_value = "12345",
        help = "Bind Port"
    )]
    port: String,

    #[structopt(
        long,
        env = "ROCKSDB_TTL",
        short,
        help = "Time-to-live (TTL) for database entries in seconds"
    )]
    ttl: Option<u64>,

    #[structopt(
        long,
        env = "ROCKSDB_TOKEN",
        help = "Authentication token for server access"
    )]
    token: Option<String>,

    #[structopt(
        long,
        env = "ROCKSDB_LOCK_FILE",
        parse(from_os_str),
        help = "Path to the lock file"
    )]
    lock_file: Option<PathBuf>,

    #[structopt(long, possible_values = &LogLevel::variants(), case_insensitive = true, env = "ROCKSDB_LOG_LEVEL", default_value = "info", help = "Logging level")]
    log_level: LogLevel,

    #[structopt(long, env = "ROCKSDB_CACHE", help = "Enable cache layer")]
    cache: bool,

    #[structopt(
        long,
        env = "ROCKSDB_CACHE_TTL",
        default_value = "1800",
        help = "Cache time-to-live in seconds"
    )]
    cache_ttl: u64,
}

#[async_std::main]
async fn main() {
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
    let cache = opt.cache;
    let cache_ttl = opt.cache_ttl;

    let lock_guard = if let Some(lock_file_path) = opt.lock_file {
        Some(create_lock_guard(lock_file_path.into()).await.unwrap())
    } else {
        None
    };

    let log_level: log::LevelFilter = opt.log_level.into();

    env_logger::Builder::new()
        .filter(None, log_level)
        .target(env_logger::Target::Stdout)
        .init();

    let addr = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    let server = Arc::new(RocksDBServer::new(dbpath, ttl, token, Some(cache_ttl), cache).unwrap());

    warn!("Server listening on {}", addr);

    let (signal_sender, signal_receiver) = bounded(1);
    ctrlc::set_handler(move || {
        let _ = signal_sender.try_send(());
    })
    .expect("Error setting Ctrl-C handler");

    let server_task = task::spawn(handle_incoming_connections(listener, server));
    let signal_task = task::spawn(handle_signals(signal_receiver));

    futures::select! {
        _ = server_task.fuse() => (),
        _ = signal_task.fuse() => (),
    }

    drop(lock_guard);

    info!("Server has shut down gracefully");
}

async fn handle_incoming_connections(listener: TcpListener, server: Arc<RocksDBServer>) {
    listener
        .incoming()
        // .for_each_concurrent(Some(1000), |stream| { // Limit concurrency to 1000
        .for_each_concurrent(/* limit */ None, |stream| {
            // Limit concurrency to 1000
            let server = server.clone();
            async move {
                match stream {
                    Ok(stream) => {
                        task::spawn(handle_connection(stream, server));
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        })
        .await;
}

async fn handle_signals(signal_receiver: Receiver<()>) {
    let _ = signal_receiver.recv().await;
    info!("Ctrl+C received, shutting down");
}

async fn handle_connection(
    socket: TcpStream,
    server: Arc<RocksDBServer>,
) -> async_std::io::Result<()> {
    let reader = BufReader::new(&socket);
    let mut writer = BufWriter::new(&socket);
    let mut lines = reader.lines();

    while let Some(line) = lines.next().await {
        match line {
            Ok(buffer) => {
                let request: Request = match serde_json::from_str(&buffer) {
                    Ok(req) => req,
                    Err(e) => {
                        error!("Failed to parse request: {}", e);
                        continue;
                    }
                };

                let response = server.handle_request(request).await;
                let response_data = serde_json::to_vec(&response).unwrap();

                if writer.write_all(&response_data).await.is_err() {
                    error!("Failed to write to socket");
                    break;
                }
                if writer.write_all(b"\n").await.is_err() {
                    error!("Failed to write to socket");
                    break;
                }
                if writer.flush().await.is_err() {
                    error!("Failed to flush socket");
                    break;
                }
            }
            Err(e) => {
                error!("Failed to read from socket: {}", e);
                return Err(e);
            }
        }
    }

    Ok(())
}
