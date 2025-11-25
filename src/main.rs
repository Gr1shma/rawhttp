use anyhow::Result;
use rawhttp::http::{Request, Response, StatusCode, body::Body, method::Method};
use rawhttp::server::{Handler, Server};

struct WebsiteHandler;

impl Handler for WebsiteHandler {
    fn handle(&self, request: &Request) -> Response {
        match request.method() {
            Method::GET => match request.path() {
                "/check" => Response::ok().with_body(Body::from("Server is running".to_string())),
                "/hi" => Response::ok().with_body(Body::from("Hello World".to_string())),
                "/smile" => {
                    let message = request.query().get("message").unwrap_or("");
                    if message.is_empty() {
                        Response::ok().with_body(Body::from("Smiling".to_string()))
                    } else {
                        Response::ok().with_body(Body::from(format!("Smiling with {}", message)))
                    }
                }
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
