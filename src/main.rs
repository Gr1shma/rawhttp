use anyhow::Result;
use rawhttp::http::{body::Body, method::Method, Request, Response, StatusCode};
use rawhttp::server::{Handler, Server};

struct WebsiteHandler;

impl Handler for WebsiteHandler {
    fn handle(&self, request: &Request) -> Response {
        match request.method() {
            Some(Method::GET) => match request.target() {
                Some("/check") => Response::ok().with_body(Body::from("Server is running".to_string())),
                Some("/hi") => Response::ok().with_body(Body::from("Hello World".to_string())),
                _ => Response::new(StatusCode::BadRequest),
            },
            _ => Response::new(StatusCode::BadRequest),
        }
    }
}

fn main() -> Result<()> {
    println!("rawhttp Server");

    let server = Server::new("127.0.0.1:8080".to_string(), WebsiteHandler);
    server.run()?;

    Ok(())
}
