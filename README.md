# rawhttp

A lightweight HTTP/1.1 parser and server built in Rust from scratch, with a focus on security and understanding how web servers work.

## Table of Contents

- [Features](#features)
- [Build & Run](#build--run)
- [Example Endpoints](#example-endpoints)
- [Testing](#testing)
- [Dependencies](#dependencies)
- [Project Structure](#project-structure)
- [Process](#process)
- [What I Have Learned](#what-i-have-learned)
- [License](#license)

## Features

### HTTP Protocol
- Full HTTP/1.1 support with chunked transfer encoding
- Parses HTTP requests including headers, body, and query parameters
- Clean error handling with helpful error messages

### Performance
- Handles multiple connections at the same time using threads

### Security
- Built-in security against request smuggling and DoS attacks
- Header size limits and connection timeouts to prevent abuse
- Host header validation to block malicious requests

## Build & Run

Ensure you have Rust installed.

To build the project:
```bash
cargo build
```

To run the server:
```bash
cargo run
```

The server will start listening on `127.0.0.1:8080`.

## Example Endpoints

Once the server is running, you can test the following endpoints:

- `GET /` - Returns "Hello from rawhttp"
- `GET /status` - Returns "Server is running"
- `GET /query?message=hello` - Returns "Message: hello"
- `POST /echo` - Echoes the request body
- `GET /valid-host` - Validates Host header against whitelist

## Testing

To run the automated handler tests, use the provided shell script:

```bash
./test_handlers.sh
```

This script sends various requests to the server and verifies the response status codes.

## Dependencies

The project uses the following external crates:

- [anyhow](https://crates.io/crates/anyhow): Flexible concrete Error type built on `std::error::Error`.
- [thiserror](https://crates.io/crates/thiserror): Convenient derivation of the `Error` trait.


## Project Structure

The project is organized into modular components:

- **`src/main.rs`**: Entry point. Defines the `WebsiteHandler` which implements the application logic and routing.
- **`src/server.rs`**: Contains the `Server` struct and `Handler` trait. Manages the TCP listener and incoming connections.
- **`src/http/`**: Library module for HTTP parsing.
  - **`request.rs`**: Parses raw bytes into `Request` structs.
  - **`response.rs`**: Formats `Response` structs into bytes.
  - **`method.rs`**, **`query.rs`**, **`body.rs`**: Helper modules for specific HTTP components.

## Process

1.  **TCP Listener**: The server binds to a TCP address and listens for incoming connections.
2.  **Connection Handling**: For each connection, a new thread is spawned (basic multi-threading).
3.  **Request Parsing**: The raw byte stream is read and parsed into a structured `Request` object.
4.  **Routing**: The `Handler` (implemented in `main.rs`) matches the request method and path to the appropriate logic.
5.  **Response Generation**: A `Response` object is created and written back to the TCP stream.

## What I Have Learned

-   How to use `std::net::TcpListener` to accept connections and `TcpStream` to read and write data.
-   Understanding the structure of HTTP requests and how to parse them manually.
-   Implementing chunked transfer encoding with proper validation.
-   Preventing request smuggling attacks by detecting duplicate Transfer-Encoding headers.
-   Protecting against DoS attacks using header size limits and connection timeouts.
-   Managing ownership when passing streams to threads and sharing handlers using `Arc`.
-   Defining a `Handler` trait to separate server infrastructure from application logic.
-   Using `anyhow` and `thiserror` for error handling in Rust.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
