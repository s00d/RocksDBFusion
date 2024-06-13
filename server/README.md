# RocksDB Server

## Overview

This project provides a simple server implementation for RocksDB, a persistent key-value store for fast storage environments. The server is designed to handle database operations over TCP connections, allowing clients to interact with the RocksDB instance remotely.

## Features

- **CRUD Operations**: Create, Read, Update, and Delete operations on RocksDB.
- **Batch Operations**: Support for batch write operations.
- **Transactions**: Begin, commit, and rollback transactions.
- **Column Families**: Manage column families within the RocksDB instance.
- **Backup and Restore**: Create and restore backups of the database.
- **Logging**: Configurable logging levels.
- **Authentication**: Optional token-based authentication for server access.
- **Custom Merge Operator**: A JSON merge operator to handle JSON merge operations.

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (latest stable version)
- RocksDB libraries

### Installation

#### From Source

1. Clone the repository:
   ```sh
   git clone https://github.com/your-repo/rocksdb-server.git
   cd rocksdb-server
   ```

2. Build the project:
   ```sh
   cargo build --release
   ```

#### Using Homebrew

1. Tap the repository:
   ```sh
   brew tap s00d/rocksdbserver
   ```

2. Install RocksDBFusion Server:
   ```sh
   brew install rocksdb_server
   ```

### Running the Server

To start the RocksDB server, use the following command:

```sh
rocksdb_server --dbpath ./db_test --port 12345 --host 127.0.0.1 --log-level info
```

### Command-Line Options

- `--dbpath <PATH>`: Path to the RocksDB database (default: `./db_test`)
- `--port <PORT>`: Port to listen on (default: `12345`)
- `--host <HOST>`: Host to bind the server to (default: `127.0.0.1`)
- `--ttl <TTL>`: Time-to-live (TTL) for database entries in seconds
- `--token <TOKEN>`: Authentication token for server access
- `--log-level <LEVEL>`: Logging level (debug, info, warn, error)
- `--lock-file <FILE>`: Path to the lock file

### macOS Sign

```bash
chmod +x ./server-0.1.1-aarch64-apple-darwin
xattr -cr ./server-0.1.1-aarch64-apple-darwin && codesign --force --deep --sign - ./server-0.1.1-aarch64-apple-darwin
```

## Code Structure

### `main.rs`

The entry point of the application. It handles command-line arguments, initializes logging, sets up the server, and listens for incoming TCP connections.

### `db_manager.rs`

Contains the `RocksDBManager` struct, which encapsulates the functionality for interacting with the RocksDB instance. It provides methods for performing CRUD operations, batch operations, transaction management, backup and restore, and more.

### `server.rs`

Defines the `RocksDBServer` struct, which manages the server's state and handles client requests. It includes methods for processing different types of requests (e.g., `put`, `get`, `delete`, `merge`) and responding to clients.

### `helpers.rs`

Provides utility functions and types, such as logging levels and lock file management.

## Examples

### Putting a Key-Value Pair

To put a key-value pair into the database:

```json
{
  "action": "put",
  "key": "example_key",
  "value": "example_value"
}
```

### Getting a Key-Value Pair

To get a value for a given key:

```json
{
  "action": "get",
  "key": "example_key"
}
```

### Deleting a Key-Value Pair

To delete a key-value pair:

```json
{
  "action": "delete",
  "key": "example_key"
}
```

### Merging a JSON Value

To merge a JSON value into an existing key:

```json
{
  "action": "merge",
  "key": "example_key",
  "value": "{\"new_field\": \"new_value\"}"
}
```

## Logging

The server uses the `env_logger` crate for logging. The logging level can be set via the command-line argument `--log-level`. Available levels are: `debug`, `info`, `warn`, `error`.

## Authentication

If the server is started with an authentication token (`--token <TOKEN>`), clients must include this token in their requests to access the server. Example request with token:

```json
{
  "action": "get",
  "key": "example_key",
  "token": "your_token"
}
```

## Backup and Restore

### Creating a Backup

To create a backup of the database:

```json
{
  "action": "backup"
}
```

### Restoring from the Latest Backup

To restore the database from the latest backup:

```json
{
  "action": "restore_latest"
}
```

### Restoring from a Specific Backup

To restore the database from a specific backup by ID:

```json
{
  "action": "restore",
  "backup_id": 1
}
```

### Getting Backup Info

To get information about available backups:

```json
{
  "action": "get_backup_info"
}
```

## Transactions

### Beginning a Transaction

To begin a new transaction:

```json
{
  "action": "begin_transaction"
}
```

### Committing a Transaction

To commit a transaction:

```json
{
  "action": "commit_transaction",
  "txn_id": 1
}
```

### Rolling Back a Transaction

To roll back a transaction:

```json
{
  "action": "rollback_transaction",
  "txn_id": 1
}
```

## Homebrew Tap for RocksDBFusion Server

This repository contains the Homebrew formula for installing RocksDBFusion Server.

### How to Use

First, you need to tap this repository:

```sh
brew tap s00d/rocksdbserver
```

Once the repository is tapped, you can install RocksDBFusion Server with the following command:

```sh
brew install rocksdb_server
```

After installation, you can start the server with:

```sh
rocksdb_server --dbpath ./db_test --port 12345 --host 127.0.0.1 --log-level info
```

Or start it as a service with:

```sh
brew services start rocksdb_server
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes.