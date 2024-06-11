# RocksDB Viewer

A simple Tauri application to view and interact with a RocksDB database. This application allows you to open a RocksDB database, view the list of keys, and see the values associated with those keys. It also supports pagination and search functionality.

## Features

- Open a RocksDB database
- View the list of keys
- See the values associated with keys
- Pagination for large datasets
- Search functionality

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (version 14 or higher)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites#installing-tauri-cli)

## Getting Started

### Clone the Repository

```bash
git clone https://github.com/s00d/rocksdb-viewer.git
cd rocksdb-viewer
```

### Install Dependencies

```bash
npm install
```

### Run the Application

```bash
npm run tauri dev
```

### Build the Application

```bash
npm run tauri build
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
