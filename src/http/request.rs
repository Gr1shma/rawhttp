use std::str;

use thiserror::Error;

use crate::http::body::{Body, BodyParseError};
use crate::http::header::Headers;

use super::header::HeaderError;
use super::method::Method;
use super::request_line::{RequestLine, RequestLineError};

#[derive(Debug, PartialEq)]
enum ParserState {
    StateInit,
    StateHeaders,
    StateBody,
    StateDone,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid request line: {0}")]
    RequestLineError(#[from] RequestLineError),

    #[error("Invalid UTF-8 encoding in request")]
    InvalidEncoding(#[from] std::str::Utf8Error),

    #[error("Incomplete or empty request")]
    IncompleteRequest,

    #[error("IO error while reading request")]
    IoError(#[from] std::io::Error),

    #[error("Invalid header")]
    InvalidHeader(#[from] HeaderError),

    #[error("Body error: {0}")]
    Body(#[from] BodyParseError),
}

pub struct Request {
    state: ParserState,
    requestline: Option<RequestLine>,
    headers: Headers,
    body: Body,
}

impl Request {
    pub fn new() -> Self {
        Request {
            state: ParserState::StateInit,
            requestline: None,
            headers: Headers::new(),
            body: Body::Empty,
        }
    }

    pub fn parse(&mut self, data: &str) -> Result<(), ParseError> {
        let (header_section, body_section) = if let Some(pos) = data.find("\r\n\r\n") {
            (&data[..pos], &data[pos + 4..])
        } else {
            (data, "")
        };

        let lines: Vec<&str> = header_section.lines().collect();
        if lines.is_empty() {
            return Err(ParseError::IncompleteRequest);
        }

        match self.state {
            ParserState::StateInit => {
                self.requestline = Some(RequestLine::parse(lines[0])?);
                self.state = ParserState::StateHeaders;

                if lines.len() > 1 {
                    self.parse_headers(&lines[1..])?;
                }

                self.state = ParserState::StateBody;

                if !body_section.is_empty() {
                    self.parse_body(body_section.as_bytes())?;
                } else {
                    if let Some(content_length) = self.header("Content-Length") {
                        if content_length != "0" {
                            return Err(ParseError::IncompleteRequest);
                        }
                    }
                }
                self.state = ParserState::StateDone;

                Ok(())
            }
            ParserState::StateHeaders => {
                self.parse_headers(&lines)?;
                self.state = ParserState::StateBody;
                Ok(())
            }
            ParserState::StateBody => {
                self.parse_body(body_section.as_bytes())?;
                self.state = ParserState::StateDone;
                Ok(())
            }
            ParserState::StateDone => Ok(()),
        }
    }

    fn parse_headers(&mut self, lines: &[&str]) -> Result<(), ParseError> {
        let header_text = lines.join("\r\n");
        self.headers.parse_headers(&header_text)?;
        Ok(())
    }

    fn parse_body(&mut self, body_bytes: &[u8]) -> Result<(), ParseError> {
        if let Some(content_length_str) = self.header("Content-Length") {
            let content_length = content_length_str.parse::<usize>().map_err(|_| {
                BodyParseError::InvalidContentLength(content_length_str.to_string())
            })?;

            self.body = Body::from_content_length(body_bytes, content_length)?;
        } else {
            if body_bytes.is_empty() {
                self.body = Body::Empty;
            } else {
                self.body = Body::Empty;
            }
        }

        Ok(())
    }

    pub fn method(&self) -> Option<&Method> {
        self.requestline.as_ref().map(|rl| &rl.method)
    }

    pub fn target(&self) -> Option<&str> {
        self.requestline.as_ref().map(|rl| rl.target.as_str())
    }

    pub fn http_version(&self) -> Option<&str> {
        self.requestline.as_ref().map(|rl| rl.httpversion.as_str())
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
}

impl TryFrom<&[u8]> for Request {
    type Error = ParseError;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let data_str = str::from_utf8(data)?;
        let mut request = Request::new();
        request.parse(data_str)?;
        Ok(request)
    }
}

pub fn request_from_reader<R: std::io::Read>(reader: &mut R) -> Result<Request, ParseError> {
    const BUFFER_SIZE: usize = 8192;
    let mut buf = vec![0; BUFFER_SIZE];

    let bytes_read = reader.read(&mut buf)?;

    Request::try_from(&buf[..bytes_read])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get_request() {
        let raw = "GET /index.html HTTP/1.1";

        let request = Request::try_from(raw.as_bytes()).unwrap();

        assert_eq!(request.method(), Some(&Method::GET));
        assert_eq!(request.target(), Some("/index.html"));
    }

    #[test]
    fn test_parse_get_request_with_headers() {
        let raw = "GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: test\r\n\r\n";

        let request = Request::try_from(raw.as_bytes()).unwrap();

        assert_eq!(request.method(), Some(&Method::GET));
        assert_eq!(request.target(), Some("/index.html"));
        assert_eq!(request.header("Host"), Some("example.com"));
        assert_eq!(request.header("User-Agent"), Some("test"));
        assert_eq!(request.headers.len(), 2);
    }

    #[test]
    fn test_parse_post_request_with_body() {
        let raw = "POST /api/users HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 11\r\n\r\nhello world";

        let request = Request::try_from(raw.as_bytes()).unwrap();

        assert_eq!(request.method(), Some(&Method::POST));
        assert_eq!(request.target(), Some("/api/users"));
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

        assert_eq!(request.method(), Some(&Method::POST));
        assert!(request.body().is_empty());
    }

    #[test]
    fn test_parse_post_request() {
        let raw = "POST /api/users HTTP/1.1";

        let request = Request::try_from(raw.as_bytes()).unwrap();

        assert_eq!(request.method(), Some(&Method::POST));
        assert_eq!(request.target(), Some("/api/users"));
    }

    #[test]
    fn test_invalid_method() {
        let raw = "INVALID /path HTTP/1.1";
        let result = Request::try_from(raw.as_bytes());

        assert!(result.is_err());
        assert!(matches!(result, Err(ParseError::RequestLineError(_))));
    }

    #[test]
    fn test_invalid_protocol() {
        let raw = "GET /path HTTPS/2.0";
        let result = Request::try_from(raw.as_bytes());

        assert!(matches!(result, Err(ParseError::RequestLineError(_))));
    }
}
