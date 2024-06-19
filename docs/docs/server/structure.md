---
lang: en-US
sticky: 10
icon: fas fa-code-branch
star: true
dir:
order: 4
category:
   - SERVER
---


# Code Structure

## `main.rs`

The entry point of the application. It handles command-line arguments, initializes logging, sets up the server, and listens for incoming TCP connections.

## `db_manager.rs`

Contains the `RocksDBManager` struct, which encapsulates the functionality for interacting with the RocksDB instance. It provides methods for performing CRUD operations, batch operations, transaction management, backup and restore, and more.

## `server.rs`

Defines the `RocksDBServer` struct, which manages the server's state and handles client requests. It includes methods for processing different types of requests (e.g., `put`, `get`, `delete`, `merge`) and responding to clients.

## `helpers.rs`

Provides utility functions and types, such as logging levels and lock file management.


## `cache.rs`

The `cache` module provides in-memory caching capabilities to improve read performance and reduce the load on RocksDB. It includes logic for managing cache entries, performing cleanup, and synchronizing with the database.

## `queue.rs`

The `queue` module implements a task queue for handling asynchronous write operations to RocksDB. This ensures that write operations are performed in a non-blocking manner, improving the overall performance of the server.
