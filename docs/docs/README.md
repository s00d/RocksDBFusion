---
home: true
title: RocksDBFusion
heroImage: /app.png
actions:
  - text: Get Started
    link: /server/
    type: primary

  - text: Introduction
    link: /introduction.html
    type: secondary

features:
  - title: 💼 Unified Management
    details: Provides a cross-platform server and multiple clients for efficient interaction with RocksDB across different platforms.
  - title: 🌐 Multi-Language Support
    details: Includes clients for various languages like Go, Node.js, PHP, Python, and Rust, ensuring wide usability.
  - title: 🖥️ User Interface
    details: Offers a graphical interface, rocksdb-client-rust, for easy interaction with the server.
  - title: ⚙️ Cross-Platform
    details: The server is built and released for multiple operating systems, making it versatile.
  - title: 🤝 Community Contributions
    details: Open to contributions from the community to enhance and extend functionality.
  - title: 📚 Comprehensive Documentation
    details: Detailed documentation to help you get started and contribute effectively.

footer: MIT Licensed | Copyright © 2024-present s00d
---

# RocksDBFusion

RocksDBFusion is a comprehensive project designed to provide a unified and efficient way to manage and interact with RocksDB across different platforms and languages. It includes a cross-platform server and multiple clients that communicate with the server via TCP, which in turn exchanges data with RocksDB.

This project is perfect for anyone looking to manage RocksDB efficiently across various platforms and languages. With RocksDBFusion, you can easily set up the server and clients, facilitating seamless data exchange through a TCP connection.

```mermaid
graph LR
    A[Client] <-->|TCP Connection| B[Server]
    B <-->|Data Exchange| C[RocksDB]
    D[Viewer] <-->|TCP Connection| B
```


### Performance Table

| Task Name | ops/sec | Average Time (ns) | Margin   | Samples |
|-----------|---------|-------------------|----------|---------|
| 'put'     | 15,151  | 66,001.68         | ±1.14%   | 15,152  |
| 'get'     | 18,665  | 53,574.02         | ±0.71%   | 18,666  |
| 'delete'  | 15,040  | 66,488.36         | ±1.04%   | 15,041  |
