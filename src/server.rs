use std::{
    net::{TcpListener, TcpStream},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use anyhow::{Context, Result};

use crate::http::{
    Request,
    request::{ParseError, request_from_reader},
    response::{Response, StatusCode},
};

pub trait Handler: Send + Sync {
    fn handle(&self, request: &Request) -> Response;

    fn handle_bad_request(&self, e: &ParseError) -> Response {
        println!("Failed to parse request: {}", e);
        Response::new(StatusCode::BadRequest)
    }
}

pub struct Server<H: Handler> {
    addr: String,
    handler: Arc<H>,
    closed: Arc<AtomicBool>,
}

impl<H: Handler + 'static> Server<H> {
    pub fn new(addr: String, handler: H) -> Self {
        Server {
            addr,
            handler: Arc::new(handler),
            closed: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr)
            .context(format!("Failed to bind the address: {}", self.addr))?;

        println!("Server listening on {}", self.addr);

        for stream in listener.incoming() {
            if self.closed.load(Ordering::Relaxed) {
                break;
            }

            match stream {
                Ok(stream) => {
                    let handler = self.handler.clone();
                    thread::spawn(move || {
                        if let Err(e) = handle_connection(stream, handler) {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("Error accepting connection: {}", e),
            }
        }

        Ok(())
    }

    pub fn close(&self) {
        self.closed.store(true, Ordering::Relaxed);
    }
}

fn handle_connection(mut stream: TcpStream, handler: Arc<dyn Handler>) -> Result<()> {
    let response = match request_from_reader(&mut stream) {
        Ok(request) => {
            println!(
                "{:?} {} HTTP/{}",
                request
                    .method()
                    .unwrap_or(&crate::http::method::Method::GET),
                request.target().unwrap_or("/"),
                request.http_version().unwrap_or("1.1")
            );
            handler.handle(&request)
        }
        Err(e) => handler.handle_bad_request(&e),
    };

    if let Err(e) = response.send(&mut stream) {
        eprintln!("Failed to send response: {}", e);
    }

    Ok(())
}
