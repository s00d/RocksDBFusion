# RocksDB Client

This repository provides a Rust implementation of a client for interacting with a RocksDB server. The client supports various operations such as put, get, delete, merge, and management of column families, transactions, and backups.

## Overview

The RocksDB Client allows you to communicate with a RocksDB server using a TCP connection. It is designed to be efficient and easy to use, making it suitable for various applications that require persistent key-value storage.

## Features

- **Basic Operations**: Perform standard operations like put, get, delete, and merge.
- **Column Family Management**: Create, list, and drop column families.
- **Transaction Management**: Begin, commit, and rollback transactions.
- **Backup and Restore**: Create and restore backups of your database.
- **Iterator Support**: Create and manage iterators to traverse database entries.

## Documentation

For detailed documentation and examples, please visit our [official documentation](https://s00d.github.io/RocksDBFusion/)).

## License

This project is licensed under the MIT License.