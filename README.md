# RocksDBFusion

RocksDBFusion is a comprehensive project that includes a server for RocksDB, multiple clients for different languages, and a client with an interface for interacting with the server. This project aims to provide a unified and efficient way to manage and interact with RocksDB across different platforms and languages.

## Project Structure

The project is organized into several workspaces, each serving a distinct purpose:

- **Server**: The main server for RocksDB, located in the `server` directory.
- **UI**: A client with a graphical interface for interacting with the server, located in the `rocksdb-viewer` directory.
- **PHP Client**: A client for PHP, located in the `client-php` directory.

## Getting Started

### Prerequisites

To build and run the projects, ensure you have the following tools installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (for the UI)
- [PHP](https://www.php.net/downloads.php) (for the PHP client)

### Building the Server

Navigate to the `server` directory and run the following commands:

```bash
cd server
cargo build --release
```

### Running the Server

After building the server, you can run it with:

```bash
cargo run --release --manifest-path=server/Cargo.toml
```

### Building the UI

Navigate to the `rocksdb-viewer` directory and run the following commands:

```bash
cd rocksdb-viewer
npm install
npm run build
```

### Running the UI

To start the UI in development mode, run:

```bash
npm start
```

### Using the PHP Client

Navigate to the `client-php` directory. You can include this client in your PHP project by requiring the necessary files. Make sure you have Composer installed to manage dependencies.

```bash
cd client-php
composer install
```

Include the client in your PHP code:

```php
require 'vendor/autoload.php';

use RocksDBFusion\Client;

// Your code here
```

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

---

RocksDBFusion aims to provide a versatile and robust solution for managing and interacting with RocksDB. We hope you find it useful and look forward to your contributions!