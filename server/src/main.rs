mod cache;
pub mod db_manager;
mod helpers;
pub mod server;
mod metrics;

use async_std::channel::{bounded, Receiver};
use async_std::io::{prelude::*, BufReader, BufWriter};
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::Arc;
use async_std::task;
use futures::stream::StreamExt;
use futures::FutureExt;
use log::{error, info, warn};
use std::env;
use std::path::PathBuf;
use std::time::{Instant};
use structopt::StructOpt;

use crate::helpers::{create_lock_guard, LogLevel};
use crate::metrics::{METRICS, Metrics};
use crate::server::{Request, RocksDBServer};

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
        env = "ROCKSDB_ADDRESS",
        default_value = "127.0.0.1:12345",
        help = "Bind address"
    )]
    address: String,

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

    #[structopt(
        long,
        env = "ROCKSDB_METRICS",
        help = "Enable metrics server"
    )]
    metrics: bool,

    #[structopt(
        long,
        env = "ROCKSDB_HEALTH_CHECK",
        help = "Enable health check endpoint"
    )]
    health_check: bool,
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

    let address = opt.address;
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



    let addr = format!("{}",address);
    let listener = TcpListener::bind(&addr).await.unwrap();

    if opt.metrics {
        METRICS.set_enabled(true);
        METRICS.observe_request_duration(0.0);

        warn!("> Metrics listening on http://{}/metrics", addr);
    }
    if opt.health_check {
        warn!("> Health check endpoint listening on http://{}/health", addr); // Добавлен вывод для health_check
    }


    let server = Arc::new(RocksDBServer::new(dbpath, ttl, token, Some(cache_ttl), cache).unwrap());

    warn!("> Server listening on {}", addr);

    let (signal_sender, signal_receiver) = bounded(1);
    ctrlc::set_handler(move || {
        let _ = signal_sender.try_send(());
    })
    .expect("Error setting Ctrl-C handler");

    let server_task = task::spawn(handle_incoming_connections(listener, server, opt.metrics, opt.health_check));
    let signal_task = task::spawn(handle_signals(signal_receiver));

    futures::select! {
        _ = server_task.fuse() => (),
        _ = signal_task.fuse() => (),
    }

    drop(lock_guard);

    info!("Server has shut down gracefully");
}

async fn handle_incoming_connections(listener: TcpListener, server: Arc<RocksDBServer>, metrics: bool, health_check: bool) {
    listener
        .incoming()
        // .for_each_concurrent(Some(1000), |stream| { // Limit concurrency to 1000
        .for_each_concurrent(/* limit */ None, |stream| {
            // Limit concurrency to 1000
            let server = server.clone();
            async move {
                match stream {
                    Ok(stream) => {
                        task::spawn(handle_connection(stream, server, metrics, health_check));
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
    metrics: bool,
    health_check: bool,
) -> async_std::io::Result<()> {
    let mut buffer = Vec::new();
    let mut reader = BufReader::new(&socket);
    let mut writer = BufWriter::new(&socket);

    while reader.read_until(b'\n', &mut buffer).await? != 0 {
        let request_str = String::from_utf8_lossy(&buffer);
        println!("Received request: {}", request_str);

        if buffer.starts_with(b"GET /favicon.ico") {
            println!("Ignoring /favicon.ico request");
            return Ok(());
        }

        if health_check && buffer.starts_with(b"GET /health ") {
            let http_response = "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nOK";

            match writer.write_all(http_response.as_bytes()).await {
                Ok(_) => info!("Successfully wrote health check response"),
                Err(e) => error!("Failed to write health check response: {}", e),
            }
            if let Err(e) = writer.flush().await {
                error!("Failed to flush health check response: {}", e);
            }
            return Ok(());
        }

        if metrics && buffer.starts_with(b"GET /metrics ") {
            METRICS.update_system_metrics();

            let response = Metrics::gather_metrics();
            let http_response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\n\r\n{}",
                response.len(),
                response
            );

            match writer.write_all(http_response.as_bytes()).await {
                Ok(_) => info!("Successfully wrote metrics response"),
                Err(e) => error!("Failed to write metrics response: {}", e),
            }
            if let Err(e) = writer.flush().await {
                error!("Failed to flush metrics response: {}", e);
            }
            return Ok(());
        }


        let start = Instant::now();
        METRICS.inc_active_requests();
        METRICS.inc_requests();

        match serde_json::from_slice::<Request>(&buffer) {
            Ok(request) => {
                let response = server.handle_request(request.clone()).await;
                let response = match serde_json::to_vec(&response) {
                    Ok(data) => {
                        let response_size = data.len() as u64;  // Размер ответа в байтах
                        METRICS.inc_response_speed_bytes(response_size);  // Наблюдаем за размером ответа
                        data
                    },
                    Err(e) => {
                        METRICS.inc_request_failure();
                        error!(
                            "Failed to serialize response: {} request {:?}",
                            e,
                            request.clone()
                        );
                        continue;
                    }
                };

                if writer.write_all(&response).await.is_err() {
                    METRICS.inc_request_failure();
                    error!("Failed to write to socket");
                    break;
                }
                if writer.write_all(b"\n").await.is_err() {
                    METRICS.inc_request_failure();
                    error!("Failed to write to socket");
                    break;
                }
                if writer.flush().await.is_err() {
                    METRICS.inc_request_failure();
                    error!("Failed to flush socket");
                    break;
                }

                METRICS.inc_request_success();
            }
            Err(e) => {
                error!("Failed to parse request: {} - {:?}", e, &buffer);
            }
        }

        METRICS.observe_request_duration(start.elapsed().as_secs_f64());
        METRICS.dec_active_requests();
        buffer.clear();
    }

    Ok(())
}

