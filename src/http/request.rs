use std::str;

use thiserror::Error;

use super::method::Method;

#[derive(Debug, PartialEq)]
enum ParserState {
    StateInit,
    StateDone,
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),

    #[error("Invalid request line format: expected 'METHOD PATH HTTP/VERSION'")]
    InvalidRequestLine,

    #[error("Invalid or unsupported HTTP protocol version: {0}")]
    InvalidProtocol(String),

    #[error("Invalid UTF-8 encoding in request")]
    InvalidEncoding(#[from] std::str::Utf8Error),

    #[error("Incomplete or empty request")]
    IncompleteRequest,

    #[error("IO error while reading request")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct RequestLine {
    pub method: Method,
    pub httpversion: String,
    pub target: String,
}

pub struct Request {
    state: ParserState,
    requestline: Option<RequestLine>,
}

impl Request {
    pub fn new() -> Self {
        Request {
            state: ParserState::StateInit,
            requestline: None,
        }
    }
    pub fn parse(&mut self, data: &str) -> Result<(), ParseError> {
        match self.state {
            ParserState::StateInit => {
                self.parse_reqest_line(data)?;
                self.state = ParserState::StateDone;
                Ok(())
            }
            ParserState::StateDone => Ok(()),
        }
    }

    fn parse_reqest_line(&mut self, data: &str) -> Result<(), ParseError> {
        let request_line = data.lines().next().ok_or(ParseError::IncompleteRequest)?;

        let parts: Vec<&str> = request_line.split_whitespace().collect();

        if parts.len() != 3 {
            return Err(ParseError::InvalidRequestLine);
        }

        let method = parts[0].parse::<Method>()?;

        let target = parts[1].to_string();
        let httpversion = parts[2].to_string();

        let http_parts: Vec<&str> = httpversion.split("/").collect();
        if http_parts.len() != 2 || http_parts[0] != "HTTP" || http_parts[1] != "1.1" {
            return Err(ParseError::InvalidProtocol(httpversion.to_string()));
        }

        self.requestline = Some(RequestLine {
            httpversion,
            method,
            target,
        });

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
        assert!(matches!(result, Err(ParseError::InvalidMethod(_))));
    }

    #[test]
    fn test_invalid_protocol() {
        let raw = "GET /path HTTPS/2.0";
        let result = Request::try_from(raw.as_bytes());

        assert!(matches!(result, Err(ParseError::InvalidProtocol(_))));
    }
}
