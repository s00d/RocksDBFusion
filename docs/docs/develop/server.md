---
lang: en-US
icon: fas fa-server
category:
   - DEVELOP
---
# Server

## Overview

The RocksDB Server is a lightweight server implementation that provides a remote interface for interacting with a RocksDB instance. It allows clients to perform various database operations over TCP connections, ensuring efficient and reliable data management.

## Features

- **CRUD Operations**: Perform Create, Read, Update, and Delete operations on the RocksDB database.
- **Batch Operations**: Support for batch write operations to optimize performance.
- **Transactions**: Begin, commit, and rollback transactions to ensure data integrity.
- **Column Families**: Manage multiple column families within the RocksDB instance.
- **Backup and Restore**: Create and restore backups of the database to prevent data loss.
- **Logging**: Configurable logging levels for monitoring and debugging.
- **Authentication**: Optional token-based authentication for secure access.
- **Custom Merge Operator**: Implement a JSON merge operator to handle JSON merge operations.
- **Cache Layer**: Optional in-memory caching to improve read performance with configurable time-to-live (TTL).

## Dependencies

The server is built using the following dependencies:

- **async-std**: Provides asynchronous I/O and task management. Used for handling concurrent connections and performing non-blocking I/O operations.
- **futures**: Utilities for asynchronous programming. Used to work with asynchronous computations and tasks.
- **rust-rocksdb**: Bindings for RocksDB. Provides an interface to interact with RocksDB for performing database operations.
- **serde** and **serde_json**: Serialization and deserialization of JSON data. Used for converting between Rust data structures and JSON.
- **json-patch**: Apply JSON Patch operations. Used for handling JSON merge operations in the database.
- **log** and **env_logger**: Logging utilities. Used for logging messages at different levels (info, debug, error, etc.).
- **structopt**: Command-line argument parsing. Used for defining and parsing command-line arguments.
- **ctrlc**: Handling Ctrl-C signals for graceful shutdowns. Used to catch interrupt signals and perform clean-up operations.
- **num_cpus**: Retrieve the number of CPUs for performance optimization. Used to set the level of parallelism for RocksDB.

## Installation and Setup

1. **Clone the Repository**:

   ```bash
   git clone https://github.com/s00d/RocksDBFusion.git
   cd RocksDBFusion/server
   ```

2. **Build the Server**:

   ```bash
   cargo build --release
   ```

3. **Run the Server**:

   ```bash
   cargo run --release
   ```

## Configuration

The server can be configured using command-line arguments or environment variables. The available options are:

- `--dbpath`, `-d`: Path to the RocksDB database (default: `./db_test`).
- `--address <HOST:PORT>`: Host and Port to listen on (default: `127.0.0.1:12345`)

see halp

Example:

```bash
ROCKSDB_PATH=./mydb ROCKSDB_ADDRESS=127.0.0.1:12345 cargo run --release
```

## Usage

The server listens for incoming TCP connections and processes requests in JSON format. Each request must specify an `action` and may include additional parameters depending on the action.

### Example Request

```json
{
  "action": "put",
  "key": "my_key",
  "value": "my_value"
}
```

### Example Response

```json
{
  "success": true,
  "result": null,
  "error": null
}
```

## Code Structure

### Main Module

The `main.rs` file initializes the server, parses command-line arguments, and starts listening for incoming connections.

### Server Module

The `server` module contains the core logic for handling requests and interacting with the RocksDB database.

### DB Manager

The `db_manager` module provides functions for performing database operations, managing transactions, and handling backups.

### Cache Module

The `cache` module provides in-memory caching capabilities to improve read performance and reduce the load on RocksDB. It includes logic for managing cache entries, performing cleanup, and synchronizing with the database.

### Task Queue Module

The `queue` module implements a task queue for handling asynchronous write operations to RocksDB. This ensures that write operations are performed in a non-blocking manner, improving the overall performance of the server.


## Request and Response

### Request Structure

The `Request` struct defines the structure of incoming requests. It includes fields such as `action`, `key`, `value`, `cf_name`, `options`, and `token`.

### Response Structure

The `Response` struct defines the structure of outgoing responses. It includes fields such as `success`, `result`, and `error`.

## Example Actions

### Put

Inserts a key-value pair into the database.

```json
{
  "action": "put",
  "key": "my_key",
  "value": "my_value"
}
```

### Get

Retrieves the value associated with a key.

```json
{
  "action": "get",
  "key": "my_key"
}
```

### Delete

Deletes a key-value pair from the database.

```json
{
  "action": "delete",
  "key": "my_key"
}
```

### Merge

Merges a value with an existing key.

```json
{
  "action": "merge",
  "key": "my_key",
  "value": "new_value"
}
```

## Handling Requests

The server handles incoming requests by matching the `action` field and calling the corresponding function in the `RocksDBServer` struct.

### Example Handler

```rust
async fn handle_put(&self, req: Request) -> Response {
    match (req.key, req.value) {
        (Some(key), Some(value)) => {
            match self.db_manager.put(key, value, req.cf_name, req.txn) {
                Ok(_) => Response {
                    success: true,
                    result: None,
                    error: None,
                },
                Err(e) => Response {
                    success: false,
                    result: None,
                    error: Some(e),
                },
            }
        }
        _ => Response {
            success: false,
            result: None,
            error: Some("Missing key or value".to_string()),
        },
    }
}
```

## Graceful Shutdown

The server handles Ctrl-C signals for graceful shutdown, ensuring that all resources are properly released.

```rust
let (signal_sender, signal_receiver) = async_std::channel::bounded(1);
ctrlc::set_handler(move || {
    let _ = signal_sender.try_send(());
}).expect("Error setting Ctrl-C handler");

let signal_task = task::spawn(async move {
    let _ = signal_receiver.recv().await;
    info!("Ctrl+C received, shutting down");
});
```

## Logging

Logging is configured using the `env_logger` crate. The logging level can be set via command-line arguments or environment variables.

```rust
env_logger::Builder::new()
    .filter(None, log_level)
    .target(env_logger::Target::Stdout)
    .init();
```

## Conclusion

The RocksDB Server provides a robust and efficient solution for managing RocksDB instances remotely. With support for various database operations, transactions, and backup/restore functionalities, it is an ideal choice for applications requiring a reliable key-value store.

For more detailed information and examples, refer to the individual sections in the documentation.