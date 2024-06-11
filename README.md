# RocksDBFusion

RocksDBFusion is a comprehensive project that includes a server for RocksDB, multiple clients for different languages, and a client with an interface for interacting with the server. This project aims to provide a unified and efficient way to manage and interact with RocksDB across different platforms and languages.

## Project Structure

The project is organized into several workspaces, each serving a distinct purpose:

- [https://packagist.org/packages/s00d/rocksdb-client-php](client-php): A client for PHP. This client is published to Composer.
- [**rocksdb-viewer**](rocksdb-viewer): A client with a graphical interface for interacting with the server. This is built and released with the prefix `Viewer`.
- [**server**](server): The main server for RocksDB. This is built and released for multiple operating systems with the prefix `server`.

## Getting Started

### Prerequisites

To build and run the projects, ensure you have the following tools installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (for the UI)
- [PHP](https://www.php.net/downloads.php) (for the PHP client)
- [Composer](https://getcomposer.org/) (for the PHP client dependencies)

## Contributing

We welcome contributions from the community! To contribute:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Make your changes.
4. Submit a pull request with a detailed description of your changes.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For any questions or suggestions, feel free to open an issue or contact the project maintainers.
