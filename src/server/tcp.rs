use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

use anyhow::{Context, Result};

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
                    }
                }
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }

        Ok(())
    }
}

fn handle_connection(stream: &mut TcpStream) -> Result<()> {
    let mut buf = vec![0; 1024];
    let bytes_read = stream
        .read(&mut buf)
        .context("Failed to read form stream")?;

    buf.truncate(bytes_read);

    println!("Received {} bytes:", bytes_read);
    println!("{:?}", buf);

    if let Ok(request_str) = std::str::from_utf8(&buf) {
        println!("\nAs string:\n{}", request_str);
    }

    Ok(())
}
