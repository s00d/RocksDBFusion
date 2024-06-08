pub mod server;
pub mod db_manager;

use std::env;
use structopt::StructOpt;
use tokio::net::TcpListener;
use server::{RocksDBServer};
use std::path::PathBuf;

#[derive(StructOpt, Debug)]
#[structopt(name = "RocksDB Server")]
struct Opt {
    #[structopt(parse(from_os_str))]
    db_path: PathBuf,
    port: u16,
    #[structopt(default_value = "127.0.0.1")]
    host: String,
    #[structopt(long)]
    ttl: Option<u64>,
    #[structopt(long)]
    token: Option<String>,
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();

    let db_path = if opt.db_path.starts_with(".") {
        env::current_dir().unwrap().join(opt.db_path)
    } else {
        opt.db_path.clone()
    };
    let db_path = db_path.to_str().unwrap().to_string();

    let port = opt.port;
    let host = opt.host;
    let ttl = opt.ttl;
    let token = opt.token;

    let addr = format!("{}:{}", host, port);


    let server = RocksDBServer::new(db_path.clone(), ttl, token).unwrap();

    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("Server listening on {}", addr);

    while let Ok((socket, _)) = listener.accept().await {
        let server = server.clone();
        tokio::spawn(async move {
            server.handle_client(socket).await;
        });
    }
}
