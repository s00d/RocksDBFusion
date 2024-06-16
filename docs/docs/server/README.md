---
lang: en-US
title: Server
sticky: 10
icon: circle-question
star: true
dir:
  order: 1
category:
  - SERVER
---

# Server

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
