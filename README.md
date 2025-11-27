# rawhttp

A simple, lightweight HTTP server implementation in Rust, built from scratch to understand the fundamentals of TCP networking and HTTP protocol parsing.

## Table of Contents

- [Build & Run](#build--run)
- [Example Endpoints](#example-endpoints)
- [Testing](#testing)
- [Dependencies](#dependencies)
- [Project Structure](#project-structure)
- [Process](#process)
- [What I Have Learned](#what-i-have-learned)
- [License](#license)

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

- **Root**: `GET /`
  - Returns: "Hello from rawhttp"
- **Status**: `GET /status`
  - Returns: "Server is running"
- **Query Echo**: `GET /query?message=hello`
  - Returns: "Message: hello"
- **Echo Body**: `POST /echo` (with body)
  - Returns: "Echo: [your body]"

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

-   **TCP Networking**: How to use `std::net::TcpListener` to accept connections and `TcpStream` to read/write data.
-   **HTTP Protocol**: Understanding the structure of HTTP requests (Method, Path, Version, Headers, Body) and how to parse them manually.
-   **Rust Ownership & Concurrency**: Managing ownership when passing streams to threads and sharing the handler using `Arc`.
-   **Traits**: Defining a `Handler` trait to decouple the server infrastructure from the application logic.
-   **Error Handling**: Using `anyhow` and `thiserror` for robust error management in Rust.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
