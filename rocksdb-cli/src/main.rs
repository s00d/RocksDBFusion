use structopt::StructOpt;
use log::{error, info};
use rocksdb_client_rust::RocksDBClient;

#[derive(StructOpt, Debug)]
#[structopt(name = "RocksDB Cli Client", about = "A simple RocksDB cli client.")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
struct CommonOpts {
    #[structopt(long, help = "Server host", default_value = "127.0.0.1")]
    host: String,
    #[structopt(long, help = "Server port", default_value = "12345")]
    port: u16,
}

#[derive(StructOpt, Debug)]
enum Command {
    /// Store a key-value pair in the database
    Put {
        #[structopt(flatten)]
        common: CommonOpts,
        #[structopt(help = "The key to store")]
        key: String,
        #[structopt(help = "The value to store")]
        value: String,
    },
    /// Retrieve the value of a key from the database
    Get {
        #[structopt(flatten)]
        common: CommonOpts,
        #[structopt(help = "The key to retrieve")]
        key: String,
    },
    /// Delete a key from the database
    Delete {
        #[structopt(flatten)]
        common: CommonOpts,
        #[structopt(help = "The key to delete")]
        key: String,
    },
    /// Merge a value with an existing key
    Merge {
        #[structopt(flatten)]
        common: CommonOpts,
        #[structopt(help = "The key to merge with")]
        key: String,
        #[structopt(help = "The value to merge")]
        value: String,
    },
    /// List all column families in the database
    ListColumnFamilies {
        #[structopt(flatten)]
        common: CommonOpts,
    },
    /// Create a new column family
    CreateColumnFamily {
        #[structopt(flatten)]
        common: CommonOpts,
        #[structopt(help = "The name of the column family to create")]
        name: String,
    },
    /// Drop an existing column family
    DropColumnFamily {
        #[structopt(flatten)]
        common: CommonOpts,
        #[structopt(help = "The name of the column family to drop")]
        name: String,
    },
    /// Compact the database within a range
    CompactRange {
        #[structopt(flatten)]
        common: CommonOpts,
        #[structopt(help = "The start key for compaction")]
        start: Option<String>,
        #[structopt(help = "The end key for compaction")]
        end: Option<String>,
    },
    /// Begin a new transaction
    BeginTransaction {
        #[structopt(flatten)]
        common: CommonOpts,
    },
    /// Commit a transaction
    CommitTransaction {
        #[structopt(flatten)]
        common: CommonOpts,
    },
    /// Rollback a transaction
    RollbackTransaction {
        #[structopt(flatten)]
        common: CommonOpts,
    },
}

fn main() {
    let opt = Opt::from_args();

    match opt.cmd {
        Command::Put { common, key, value } => {
            let mut client = RocksDBClient::new(common.host, common.port);
            info!("Sending PUT request: key={}, value={}", key, value);
            match client.put(key, value, None, None) {
                Ok(_) => println!("PUT request successful"),
                Err(e) => error!("Failed to put value: {}", e),
            }
        }
        Command::Get { common, key } => {
            let mut client = RocksDBClient::new(common.host, common.port);
            info!("Sending GET request: key={}", key);
            match client.get(key, None, None, None) {
                Ok(Some(value)) => println!("result: {}", value),
                Ok(None) => println!("GET request successful: key not found"),
                Err(e) => error!("Failed to get value: {}", e),
            }
        }
        Command::Delete { common, key } => {
            let mut client = RocksDBClient::new(common.host, common.port);
            info!("Sending DELETE request: key={}", key);
            match client.delete(key, None, None) {
                Ok(_) => println!("DELETE request successful"),
                Err(e) => error!("Failed to delete key: {}", e),
            }
        }
        Command::Merge { common, key, value } => {
            let mut client = RocksDBClient::new(common.host, common.port);
            info!("Sending MERGE request: key={}, value={}", key, value);
            match client.merge(key, value, None, None) {
                Ok(_) => println!("MERGE request successful"),
                Err(e) => error!("Failed to merge value: {}", e),
            }
        }
        Command::ListColumnFamilies { common } => {
            let mut client = RocksDBClient::new(common.host, common.port);
            info!("Sending LIST_COLUMN_FAMILIES request");
            match client.list_column_families() {
                Ok(families) => println!("result: {:?}", families),
                Err(e) => error!("Failed to list column families: {}", e),
            }
        }
        Command::CreateColumnFamily { common, name } => {
            let mut client = RocksDBClient::new(common.host, common.port);
            info!("Sending CREATE_COLUMN_FAMILY request: name={}", name);
            match client.create_column_family(name) {
                Ok(_) => println!("CREATE_COLUMN_FAMILY request successful"),
                Err(e) => error!("Failed to create column family: {}", e),
            }
        }
        Command::DropColumnFamily { common, name } => {
            let mut client = RocksDBClient::new(common.host, common.port);
            info!("Sending DROP_COLUMN_FAMILY request: name={}", name);
            match client.drop_column_family(name) {
                Ok(_) => println!("DROP_COLUMN_FAMILY request successful"),
                Err(e) => error!("Failed to drop column family: {}", e),
            }
        }
        Command::CompactRange { common, start, end } => {
            let mut client = RocksDBClient::new(common.host, common.port);
            info!("Sending COMPACT_RANGE request: start={:?}, end={:?}", start, end);
            match client.compact_range(start, end, None) {
                Ok(_) => println!("COMPACT_RANGE request successful"),
                Err(e) => error!("Failed to compact range: {}", e),
            }
        }
        Command::BeginTransaction { common } => {
            let mut client = RocksDBClient::new(common.host, common.port);
            info!("Sending BEGIN_TRANSACTION request");
            match client.begin_transaction() {
                Ok(_) => println!("BEGIN_TRANSACTION request successful"),
                Err(e) => error!("Failed to begin transaction: {}", e),
            }
        }
        Command::CommitTransaction { common } => {
            let mut client = RocksDBClient::new(common.host, common.port);
            info!("Sending COMMIT_TRANSACTION request");
            match client.commit_transaction() {
                Ok(_) => println!("COMMIT_TRANSACTION request successful"),
                Err(e) => error!("Failed to commit transaction: {}", e),
            }
        }
        Command::RollbackTransaction { common } => {
            let mut client = RocksDBClient::new(common.host, common.port);
            info!("Sending ROLLBACK_TRANSACTION request");
            match client.rollback_transaction() {
                Ok(_) => println!("ROLLBACK_TRANSACTION request successful"),
                Err(e) => error!("Failed to rollback transaction: {}", e),
            }
        }
    }
}
