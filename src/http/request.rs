use std::io::{BufRead, BufReader, Read};
use std::str;

use thiserror::Error;

use crate::http::body::{Body, BodyError};
use crate::http::header::Headers;

use super::header::HeaderError;
use super::method::Method;
use super::request_line::{RequestLine, RequestLineError};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid request line: {0}")]
    RequestLine(#[from] RequestLineError),

    #[error("Invalid UTF-8 encoding in request")]
    InvalidEncoding(#[from] std::str::Utf8Error),

    #[error("Incomplete or empty request")]
    IncompleteRequest,

    #[error("IO error while reading request")]
    IoError(#[from] std::io::Error),

    #[error("Invalid header")]
    Header(#[from] HeaderError),

    #[error("Body error: {0}")]
    Body(#[from] BodyError),
}

pub struct Request {
    pub requestline: RequestLine,
    pub headers: Headers,
    pub body: Body,
}

impl Request {
    pub fn method(&self) -> &Method {
        &self.requestline.method
    }

    pub fn target(&self) -> &str {
        self.requestline.target.as_str()
    }

    pub fn http_version(&self) -> &str {
        self.requestline.httpversion.as_str()
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)
    }

    pub fn body(&self) -> &Body {
        &self.body
    }

    pub fn body_as_bytes(&self) -> &[u8] {
        self.body.as_bytes()
    }

    pub fn body_as_str(&self) -> Result<&str, std::str::Utf8Error> {
        self.body.as_str()
    }

    pub fn from_parts(header_section: &str, body: Vec<u8>) -> Result<Self, ParseError> {
        let lines: Vec<&str> = header_section.lines().collect();
        if lines.is_empty() {
            return Err(ParseError::IncompleteRequest);
        }

        let requestline = RequestLine::parse(lines[0])?;
        let mut headers = Headers::new();

        if lines.len() > 1 {
            let header_text = lines[1..].join("\r\n");
            headers.parse_headers(&header_text)?;
        }

        let body = if let Some(content_length_str) = headers.get("Content-Length") {
            let content_length = content_length_str
                .parse::<usize>()
                .map_err(|_| BodyError::InvalidContentLength(content_length_str.to_string()))?;

            Body::from_content_length(&body, content_length)?
        } else if body.is_empty() {
            Body::Empty
        } else {
            Body::Content(body)
        };

        Ok(Request {
            requestline,
            headers,
            body,
        })
    }
}

impl TryFrom<&[u8]> for Request {
    type Error = ParseError;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let data_str = str::from_utf8(data)?;

        let (header_section, body_section) = if let Some(pos) = data_str.find("\r\n\r\n") {
            (&data_str[..pos], &data_str[pos + 4..])
        } else {
            (data_str, "")
        };

        Request::from_parts(header_section, body_section.as_bytes().to_vec())
    }
}

pub fn request_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Request, ParseError> {
    let mut reader = BufReader::new(reader);
    let mut headers_buf = Vec::new();
    let mut content_length = 0;

    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break; // EOF
        }

        if line == "\r\n" || line == "\n" {
            break; // End of headers
        }

        headers_buf.extend_from_slice(line.as_bytes());

        if line.to_lowercase().starts_with("content-length:") {
            if let Some(value) = line.split(':').nth(1) {
                if let Ok(len) = value.trim().parse::<usize>() {
                    content_length = len;
                }
            }
        }
    }

    let mut body_buf = vec![0; content_length];
    reader.read_exact(&mut body_buf)?;

    let headers_str =
        String::from_utf8(headers_buf).map_err(|e| ParseError::InvalidEncoding(e.utf8_error()))?;

    Request::from_parts(&headers_str, body_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get_request() {
        let raw = "GET /index.html HTTP/1.1";

        let request = Request::try_from(raw.as_bytes()).unwrap();

        assert_eq!(request.method(), &Method::GET);
        assert_eq!(request.target(), "/index.html");
    }

    #[test]
    fn test_parse_get_request_with_headers() {
        let raw = "GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: test\r\n\r\n";

        let request = Request::try_from(raw.as_bytes()).unwrap();

        assert_eq!(request.method(), &Method::GET);
        assert_eq!(request.target(), "/index.html");
        assert_eq!(request.header("Host"), Some("example.com"));
        assert_eq!(request.header("User-Agent"), Some("test"));
        assert_eq!(request.headers.len(), 2);
    }

    #[test]
    fn test_parse_post_request_with_body() {
        let raw = "POST /api/users HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 11\r\n\r\nhello world";

        let request = Request::try_from(raw.as_bytes()).unwrap();

        assert_eq!(request.method(), &Method::POST);
        assert_eq!(request.target(), "/api/users");
        assert_eq!(request.header("Content-Type"), Some("application/json"));
        assert_eq!(request.header("Content-Length"), Some("11"));
        assert_eq!(request.body_as_str().unwrap(), "hello world");
        assert_eq!(request.body().len(), 11);
        assert!(!request.body().is_empty());
    }

    #[test]
    fn test_parse_post_request_without_body() {
        let raw = "POST /api/users HTTP/1.1\r\n\r\n";

        let request = Request::try_from(raw.as_bytes()).unwrap();

        assert_eq!(request.method(), &Method::POST);
        assert!(request.body().is_empty());
    }

    #[test]
    fn test_parse_post_request() {
        let raw = "POST /api/users HTTP/1.1";

        let request = Request::try_from(raw.as_bytes()).unwrap();

        assert_eq!(request.method(), &Method::POST);
        assert_eq!(request.target(), "/api/users");
    }

    #[test]
    fn test_invalid_method() {
        let raw = "INVALID /path HTTP/1.1";
        let result = Request::try_from(raw.as_bytes());

        assert!(result.is_err());
        assert!(matches!(result, Err(ParseError::RequestLine(_))));
    }

    #[test]
    fn test_invalid_protocol() {
        let raw = "GET /path HTTPS/2.0";
        let result = Request::try_from(raw.as_bytes());

        assert!(matches!(result, Err(ParseError::RequestLine(_))));
    }

    #[test]
    fn test_large_request_body() {
        let body_size = 10 * 1024; // 10KB
        let body = "a".repeat(body_size);
        let raw = format!(
            "POST /large HTTP/1.1\r\nContent-Length: {}\r\n\r\n{}",
            body_size, body
        );

        let mut cursor = std::io::Cursor::new(raw.as_bytes());
        let request = request_from_reader(&mut cursor).unwrap();

        assert_eq!(request.body().len(), body_size);
        assert_eq!(request.body_as_str().unwrap(), body);
    }
}
