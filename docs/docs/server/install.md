---
lang: en-US
title: Install
sticky: 10
icon: fas fa-download
star: true
dir:
order: 2
category:
   - SERVER
---


# Getting Started

## Prerequisites

- Rust (latest stable version)
- Cargo (latest stable version)
- RocksDB libraries

## Installation

### From Source

1. Clone the repository:
   ```sh
   git clone https://github.com/your-repo/rocksdb-server.git
   cd rocksdb-server
   ```

2. Build the project:
   ```sh
   cargo build --release
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
rocksdb_server --dbpath ./db_test --address 127.0.0.1:12345 --host 127.0.0.1 --log-level info
```

Or start it as a service with:

```sh
export ROCKSDB_PATH="$(brew --prefix)/var/rocksdb/db"
export ROCKSDB_ADDRESS=127.0.0.1:12345
export ROCKSDB_LOCK_FILE="$(brew --prefix)/var/rocksdb/rocksdb.lock"

brew services start rocksdb_server
```

### Using Snap (Linux)

RocksDB Server is available as a Snap package for easy installation on Linux systems.

1. Install Snapd (if not already installed):
   ```sh
   sudo apt update
   sudo apt install snapd
   ```

2. Install RocksDB Server:
   ```sh
   sudo snap install rocksdb-server
   ```

3. Start the server:
   ```sh
   rocksdb-server.rocksdb-server --dbpath ./db_test --address 127.0.0.1:12345 --host 127.0.0.1 --log-level info
   ```

4. Enable and start as a service:
   ```sh
   sudo snap start rocksdb-server
   ```

### Finding Logs

To find the logs for the RocksDB server, use the following command:

```sh
tail -f $(brew --prefix)/var/log/rocksdb_server.log
```

### Finding the Database Directory

The RocksDB server stores its data in the `var/rocksdb/db` directory under the Homebrew prefix. To find and navigate to this directory, use the following commands:

1. Find the Homebrew prefix:
   ```sh
   brew --prefix
   ```

   This will output the Homebrew prefix, for example, `/usr/local`.

2. Navigate to the `rocksdb` data directory:
   ```sh
   cd $(brew --prefix)/var/rocksdb
   ```

### macOS Sign

If you are on macOS, you may need to sign the application before running it. Here are the steps:

1. Make the binary executable:

    ```bash
    chmod +x ./server-0.1.2-aarch64-apple-darwin
    ```

2. Clear extended attributes and sign the binary:

    ```bash
    xattr -cr ./server-0.1.2-aarch64-apple-darwin && codesign --force --deep --sign - ./server-0.1.2-aarch64-apple-darwin
    ```

### Using systemd on Linux

1. Create a systemd service file:

   ```sh
   sudo nano /etc/systemd/system/rocksdb_server.service
   ```

2. Add the following content to the service file:

   ```ini
   [Unit]
   Description=RocksDB Server
   After=network.target

   [Service]
   Environment="ROCKSDB_PATH=${ROCKSDB_PATH:-/var/rocksdb/db}"
   Environment="ROCKSDB_PORT=${ROCKSDB_ADDRESS:-127.0.0.1:12345}"
   Environment="ROCKSDB_LOCK_FILE=${ROCKSDB_LOCK_FILE:-/var/rocksdb/rocksdb.lock}"
   ExecStart=/usr/local/bin/rocksdb_server --dbpath $ROCKSDB_PATH --address $ROCKSDB_PORT --lock-file $ROCKSDB_LOCK_FILE --host 127.0.0.1 --log-level info
   Restart=always
   User=nobody
   Group=nogroup

   [Install]
   WantedBy=multi-user.target
   ```

3. Reload systemd to apply the new service:

   ```sh
   sudo systemctl daemon-reload
   ```

4. Enable and start the RocksDB server service:

   ```sh
   sudo systemctl enable rocksdb_server
   sudo systemctl start rocksdb_server
   ```

### Running the Server

To start the RocksDB server, use the following command:

```sh
rocksdb_server --dbpath ./db_test --address 127.0.0.1:12345 --host 127.0.0.1 --log-level info
```

### Command-Line Options and Environment Variables

- `--dbpath <PATH>`: Path to the RocksDB database (default: `./db_test`, env: `ROCKSDB_PATH`)
- `--address <HOST:PORT>`: Host and Port to listen on (default: `127.0.0.1:12345`, env: `ROCKSDB_ADDRESS`)
- `--ttl <TTL>`: Time-to-live (TTL) for database entries in seconds (env: `ROCKSDB_TTL`)
- `--token <TOKEN>`: Authentication token for server access (env: `ROCKSDB_TOKEN`)
- `--log-level <LEVEL>`: Logging level (debug, info, warn, error) (default: `info`, env: `ROCKSDB_LOG_LEVEL`)
- `--lock-file <FILE>`: Path to the lock file (env: `ROCKSDB_LOCK_FILE`)
- `--cache`: Enable cache layer (default: `false`, env: `ROCKSDB_CACHE`)
- `--cache-ttl <TTL>`: Cache time-to-live in seconds (default: `1800`, env: `ROCKSDB_CACHE_TTL`)
- `--metrics`: Enable metrics server (default: `false`, env: `ROCKSDB_METRICS`)
- `--health-check`: Enable health check endpoint (default: `false`, env: `ROCKSDB_HEALTH_CHECK`)

see `rocksdb-server -h`

### Logging

The server uses the `env_logger` crate for logging. The logging level can be set via the command-line argument `--log-level`. Available levels are: `debug`, `info`, `warn`, `error`.

### Authentication

If the server is started with an authentication token (`--token <TOKEN>`), clients must include this token in their requests to access the server. Example request with token:
