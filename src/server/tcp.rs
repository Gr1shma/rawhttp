use std::net::{TcpListener, TcpStream};

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
                    }
                }
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }

        Ok(())
    }
}

fn handle_connection(stream: &mut TcpStream) -> Result<()> {
    let request = request_from_reader(stream)?;

    println!("{:?}", request.method());
    println!("{:?}", request.target());

    Ok(())
}
