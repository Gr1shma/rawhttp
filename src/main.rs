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
                "/valid-host" => {
                    let allowed_hosts =
                        &["localhost:8080", "127.0.0.1:8080", "grishmadhakal.com.np"];

                    if let Some(host) = request.validated_host(allowed_hosts) {
                        let api_docs_url = format!("http://{}/docs", host);
                        let reset_url = format!("http://{}/reset-password", host);

                        let response_body = format!(
                            "Host Information:\n\
                             - Documentation: {}\n\
                             - Password Reset: {}\n\
                             \n\
                             Note: Host header validated against whitelist to prevent attacks.",
                            api_docs_url, reset_url
                        );
                        Response::ok().with_body(Body::from(response_body))
                    } else {
                        Response::bad_request().with_body(Body::from(
                            "Invalid or missing Host header. Allowed hosts: localhost:8080, 127.0.0.1:8080, grishmadhakal.com.np"
                        ))
                    }
                }
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
