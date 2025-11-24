use std::fmt::Display;

use crate::http::{Headers, body::Body};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
    Ok = 200,
    BadRequest = 400,
    InternalServerError = 500,
}

impl StatusCode {
    pub fn reason_parse(&self) -> &'static str {
        match self {
            StatusCode::Ok => "OK",
            StatusCode::BadRequest => "Bad Request",
            StatusCode::InternalServerError => "Internal Server Error",
        }
    }

    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.as_u16(), self.reason_parse())
    }
}

#[derive(Debug)]
pub struct Response {
    pub status_code: StatusCode,
    pub headers: Headers,
    pub body: Body,
}

impl Response {
    pub fn new(status_code: StatusCode) -> Self {
        let mut headers = Headers::new();
        headers.insert("Connection".to_string(), "close".to_string());
        Response {
            status_code,
            headers,
            body: Body::Empty,
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::Ok)
    }

    pub fn bad_request() -> Self {
        Self::new(StatusCode::BadRequest)
    }

    pub fn internal_server_error() -> Self {
        Self::new(StatusCode::InternalServerError)
    }

    pub fn with_body(mut self, body: Body) -> Self {
        self.body = body;

        if !self.body.is_empty() {
            self.headers
                .insert("Content-Length".to_string(), self.body.len().to_string());
        }

        self
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn with_headers(mut self, headers: Headers) -> Self {
        for (name, value) in headers.iter() {
            self.headers.insert(name.to_string(), value.to_string());
        }
        self
    }

    pub fn status_code(&self) -> StatusCode {
        self.status_code
    }

    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut response = Vec::new();

        let status_line = format!("HTTPS/1.1 {}\r\n", self.status_code());
        response.extend_from_slice(status_line.as_bytes());

        for (name, value) in self.headers.iter() {
            let header_line = format!("{}: {}\r\n", name, value);
            response.extend_from_slice(header_line.as_bytes());
        }

        response.extend_from_slice(b"\r\n");

        response.extend_from_slice(self.body.as_bytes());

        return response;
    }
}

impl Default for Response {
    fn default() -> Self {
        Self::ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_code_display() {
        assert_eq!(StatusCode::Ok.to_string(), "200 OK");
        assert_eq!(StatusCode::BadRequest.to_string(), "400 Bad Request");
        assert_eq!(
            StatusCode::InternalServerError.to_string(),
            "500 Internal Server Error"
        );
    }

    #[test]
    fn test_basic_response() {
        let response = Response::ok();
        assert_eq!(response.status_code(), StatusCode::Ok);
        assert_eq!(response.body().is_empty(), true)
    }

    #[test]
    fn test_response_with_body() {
        let data = b"Hello, World!";
        let body = Body::from_content_length(data, data.len()).unwrap();
        let response = Response::ok().with_body(body);

        assert_eq!(response.status_code(), StatusCode::Ok);
        assert_eq!(response.body().as_str().unwrap(), "Hello, World!");
        assert_eq!(response.headers().get("Content-Length"), Some("13"));
    }
}
