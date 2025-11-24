use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

use anyhow::{Context, Result};

use crate::http::request::request_from_reader;

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(addr: String) -> Self {
        Server { addr }
    }
    pub fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr)
            .context(format!("Failed to bind the address: {}", self.addr))?;

        println!("Server listening on {}", self.addr);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    if let Err(e) = handle_connection(&mut stream) {
                        eprintln!("Error handling connection: {}", e);
                        let _ = send_error_response(&mut stream);
                    }
                }
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }

        Ok(())
    }
}

fn handle_connection(stream: &mut TcpStream) -> Result<()> {
    let request = request_from_reader(stream).context("Failed to parse HTTP request")?;

    println!(
        "{:?} {} HTTP/{}",
        request
            .method()
            .unwrap_or(&crate::http::method::Method::GET),
        request.target().unwrap_or("/"),
        request.http_version().unwrap_or("1.1")
    );

    for (name, value) in request.headers.iter() {
        println!("  {}: {}", name, value);
    }

    if !request.body().is_empty() {
        println!("Body: {} bytes", request.body().len());
        if let Ok(body_str) = request.body_as_str() {
            println!("  {}", body_str);
        }
    }

    send_response(stream)?;

    Ok(())
}

fn send_response(stream: &mut TcpStream) -> Result<()> {
    let body = "Hello, World!";
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/plain\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        body.len(),
        body
    );

    stream
        .write_all(response.as_bytes())
        .context("Failed to write response to stream")?;

    stream.flush().context("Failed to flush stream")?;

    Ok(())
}

fn send_error_response(stream: &mut TcpStream) -> Result<()> {
    let body = "400 Bad Request";
    let response = format!(
        "HTTP/1.1 400 Bad Request\r\n\
         Content-Type: text/plain\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        body.len(),
        body
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}
