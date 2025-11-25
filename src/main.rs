use anyhow::Result;
use rawhttp::http::{Request, Response, StatusCode, body::Body, method::Method};
use rawhttp::server::{Handler, Server};

struct WebsiteHandler;

impl Handler for WebsiteHandler {
    fn handle(&self, request: &Request) -> Response {
        match request.method() {
            Method::GET => match request.path() {
                "/" => Response::ok().with_body(Body::from("Hello from rawhttp".to_string())),
                "/status" => Response::ok().with_body(Body::from("Server is running".to_string())),
                "/query" => {
                    let message = request.query().get("message").unwrap_or("");
                    if message.is_empty() {
                        Response::ok().with_body(Body::from("No message provided".to_string()))
                    } else {
                        Response::ok().with_body(Body::from(format!("Message: {}", message)))
                    }
                }
                _ => Response::not_found().with_body(Body::from("Not found".to_string())),
            },
            Method::POST => match request.path() {
                "/echo" => {
                    let body = request.body().as_str().unwrap_or("(invalid UTF-8)");
                    Response::ok().with_body(Body::from(format!("Echo: {}", body)))
                }
                _ => Response::not_found().with_body(Body::from("Not found".to_string())),
            },
            _ => Response::new(StatusCode::MethodNotAllowed)
                .with_body(Body::from("Method not allowed".to_string())),
        }
    }
}

fn main() -> Result<()> {
    println!("rawhttp Server");

    let server = Server::new("127.0.0.1:8080".to_string(), WebsiteHandler);
    server.run()?;

    Ok(())
}
