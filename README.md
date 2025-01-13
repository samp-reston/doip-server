# Diagnostics over Internet Protocol (DoIP) Server

An open-source library for implementing a simple DoIP server, built using the `doip-sockets`, `doip-definitions`, and `doip-codec` crates. This library provides an asynchronous and modular framework for handling DoIP requests and responses in compliance with ISO 13400 standards.

## Features

- **Seamless Integration**: Leverages `doip-sockets` for networking, `doip-definitions` for protocol-specific types, and `doip-codec` for message encoding/decoding.
- **Asynchronous Design**: Built on [Tokio](https://tokio.rs/) for efficient, non-blocking operations.
- **Configurable Handlers**: Supports custom logic for handling various DoIP services.
- **Extensible and Modular**: Easily extend the server with additional functionality or integrate with larger diagnostic systems.
- **Error Handling**: Provides robust error reporting for socket operations and DoIP protocol violations.

## Getting Started

### Prerequisites

- Rust programming language (latest stable version). Install Rust from [rust-lang.org](https://www.rust-lang.org/).

### Installation

Add the library as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
doip-server = "0.1.0"
```

### Documentation

Comprehensive documentation is available [here](#) (link to hosted docs).

## Development

### Building the Project

Clone the repository and build the project using Cargo:

```sh
git clone https://github.com/samp-reston/doip-server.git
cd doip-server
cargo build
```

### Running Tests

Run unit tests to ensure functionality:

```sh
cargo test
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Based on the ISO 13400 DoIP specification.
- Built with the Rust programming language.
- Relies on `doip-sockets`, `doip-definitions`, and `doip-codec` crates.
- Thanks to the Tokio project for enabling high-performance asynchronous networking.

## Contact

For support, questions, or feature requests, please open an issue on the [GitHub repository](https://github.com/samp-reston/doip-server).
