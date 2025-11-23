use thiserror::Error;

use super::Method;

#[derive(Debug, Error)]
pub enum RequestLineError {
    #[error("Invalid HTTP method: {0}")]
    InvalidMethod(String),

    #[error("Invalid request line format: expected 'METHOD PATH HTTP/VERSION'")]
    InvalidRequestLine,

    #[error("Invalid or unsupported HTTP protocol version: {0}")]
    InvalidProtocol(String),
}

#[derive(Debug)]
pub struct RequestLine {
    pub method: Method,
    pub httpversion: String,
    pub target: String,
}

impl RequestLine {
    pub fn parse(line: &str) -> Result<Self, RequestLineError> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 3 {
            return Err(RequestLineError::InvalidRequestLine);
        }

        let method = parts[0]
            .parse::<Method>()
            .map_err(|_| RequestLineError::InvalidMethod(parts[0].to_string()))?;

        let target = parts[1].to_string();
        let httpversion = parts[2].to_string();

        let http_parts: Vec<&str> = httpversion.split("/").collect();
        if http_parts.len() != 2 || http_parts[0] != "HTTP" || http_parts[1] != "1.1" {
            return Err(RequestLineError::InvalidProtocol(httpversion.to_string()));
        }

        Ok(RequestLine {
            httpversion,
            method,
            target,
        })
    }
}
