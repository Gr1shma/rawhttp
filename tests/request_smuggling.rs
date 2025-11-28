use rawhttp::http::{Request, Response, StatusCode};
use rawhttp::server::{Handler, Server};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

struct TestHandler {
    bodies: Arc<Mutex<Vec<String>>>,
}

impl TestHandler {
    fn new() -> Self {
        TestHandler {
            bodies: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Handler for TestHandler {
    fn handle(&self, request: &Request) -> Response {
        let mut bodies = self.bodies.lock().unwrap();
        if let Ok(body_str) = request.body_as_str() {
            bodies.push(body_str.to_string());
        }
        Response::new(StatusCode::OK)
    }
}

fn start_server(port: u16) -> (Arc<Mutex<Vec<String>>>, Arc<Server<TestHandler>>) {
    let handler = TestHandler::new();
    let bodies = handler.bodies.clone();
    let server = Server::new(format!("127.0.0.1:{}", port), handler);
    let server = Arc::new(server);
    let server_clone = server.clone();

    thread::spawn(move || {
        if let Err(e) = server_clone.run() {
            eprintln!("Server error: {}", e);
        }
    });

    thread::sleep(Duration::from_millis(100));
    (bodies, server)
}

fn send_request(port: u16, request: &str) -> String {
    let mut stream = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
    stream.write_all(request.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap_or_default();
    response
}

#[test]
fn test_conflicting_cl_te() {
    let port = 8082;
    let (bodies, server) = start_server(port);

    // CL says 3 bytes ("5\r\n"), TE says chunked ("ABCDE")
    let request = "POST / HTTP/1.1\r\n\
                   Host: localhost\r\n\
                   Content-Length: 3\r\n\
                   Transfer-Encoding: chunked\r\n\
                   \r\n\
                   5\r\n\
                   ABCDE\r\n\
                   0\r\n\r\n";

    let response = send_request(port, request);
    server.close();

    let bodies = bodies.lock().unwrap();
    if bodies.is_empty() {
        assert!(
            response.contains("400 Bad Request"),
            "Should be 400 Bad Request if rejected, got: {}",
            response
        );
    } else {
        let body = &bodies[0];
        assert_ne!(
            body, "5\r\n",
            "VULNERABLE: Server used Content-Length and ignored Transfer-Encoding"
        );
    }
}

#[test]
fn test_cl_cl_vulnerability() {
    let port = 8083;
    let (_bodies, server) = start_server(port);

    let request = "POST / HTTP/1.1\r\n\
                   Host: localhost\r\n\
                   Content-Length: 5\r\n\
                   Content-Length: 6\r\n\
                   \r\n\
                   ABCDEF";

    let response = send_request(port, request);
    server.close();

    assert!(
        response.contains("400 Bad Request"),
        "Should reject double Content-Length, got: {}",
        response
    );
}

#[test]
fn test_te_te_vulnerability() {
    let port = 8084;
    let (_bodies, server) = start_server(port);

    let request = "POST / HTTP/1.1\r\n\
                   Host: localhost\r\n\
                   Content-Length: 3\r\n\
                   Transfer-Encoding: chunked\r\n\
                   Transfer-Encoding: identity\r\n\
                   \r\n\
                   5\r\n\
                   ABCDE\r\n\
                   0\r\n\r\n";

    let response = send_request(port, request);
    server.close();

    assert!(
        response.contains("400 Bad Request"),
        "Should reject double Transfer-Encoding, got: {}",
        response
    );
}
