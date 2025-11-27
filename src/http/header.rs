use std::collections::HashMap;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HeaderError {
    #[error("Invalid head format: missing colon separator")]
    MissingColon,

    #[error("Invalid header format: empty header name")]
    EmptyHeaderName,

    #[error("Invalid header format: empty header value")]
    EmptyHeaderValue,

    #[error("Invalid header name: contains invalid characters")]
    InvalidHeaderName,

    #[error("Invalid header value: contains invalid characters")]
    InvalidHeaderValue,
}

#[derive(Debug, Clone)]
pub struct Headers {
    pub headers: HashMap<String, String>,
}

impl Headers {
    pub fn new() -> Self {
        Headers {
            headers: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into();
        let value = value.into();
        let key = name.to_lowercase();

        self.headers
            .entry(key)
            .and_modify(|existing| {
                existing.push(',');
                existing.push_str(&value);
            })
            .or_insert(value);
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(|s| s.as_str())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.headers.contains_key(&name.to_lowercase())
    }

    pub fn len(&self) -> usize {
        self.headers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.headers.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    fn is_valid_token(s: &str) -> bool {
        !s.is_empty()
            && s.chars().all(|c| {
                c.is_ascii_alphanumeric()
                    || matches!(
                        c,
                        '!' | '#'
                            | '$'
                            | '%'
                            | '&'
                            | '\''
                            | '*'
                            | '+'
                            | '-'
                            | '.'
                            | '^'
                            | '_'
                            | '`'
                            | '|'
                            | '~'
                    )
            })
    }

    fn is_valid_header_value(s: &str) -> bool {
        s.chars()
            .all(|c| matches!(c, ' ' | '\t') || c.is_ascii_graphic() || !c.is_ascii())
    }

    fn parse_header_line(line: &str) -> Result<(String, String), HeaderError> {
        let (name, value) = line.split_once(':').ok_or(HeaderError::MissingColon)?;

        let name = name.trim();
        let value = value.trim();

        if name.is_empty() {
            return Err(HeaderError::EmptyHeaderName);
        }

        if value.is_empty() {
            return Err(HeaderError::EmptyHeaderValue);
        }

        if !Self::is_valid_token(name) {
            return Err(HeaderError::InvalidHeaderName);
        }

        if !Self::is_valid_header_value(value) {
            return Err(HeaderError::InvalidHeaderValue);
        }

        Ok((name.to_string(), value.to_string()))
    }

    pub fn parse_headers(&mut self, lines: &str) -> Result<(), HeaderError> {
        for line in lines.split("\r\n") {
            if line.is_empty() {
                break;
            }
            let (name, value) = Self::parse_header_line(line)?;
            self.insert(name, value);
        }
        Ok(())
    }
}

impl Default for Headers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_headers() {
        let lines = "Host: localhost:42069\r\nFoFo:     barbar\r\n\r\n";

        let mut headers = Headers::new();
        let result = headers.parse_headers(lines);

        assert!(result.is_ok());
        assert_eq!(headers.len(), 2);
        assert_eq!(headers.get("Host"), Some("localhost:42069"));
        assert_eq!(headers.get("FoFo"), Some("barbar"));
    }

    #[test]
    fn test_invalid_header_name() {
        let line = "Content Type: text/html";
        let result = Headers::parse_header_line(line);
        assert!(matches!(result, Err(HeaderError::InvalidHeaderName)));
    }

    #[test]
    fn test_invalid_header_value() {
        let line = "Content-Type: text/html\0";
        let result = Headers::parse_header_line(line);
        assert!(matches!(result, Err(HeaderError::InvalidHeaderValue)));
    }

    #[test]
    fn test_missing_colon() {
        let line = "InvalidHeader";
        let result = Headers::parse_header_line(line);
        assert!(matches!(result, Err(HeaderError::MissingColon)));
    }

    #[test]
    fn test_empty_header_name() {
        let line = ": value";
        let result = Headers::parse_header_line(line);
        assert!(matches!(result, Err(HeaderError::EmptyHeaderName)));
    }

    #[test]
    fn test_empty_header_value() {
        let line = "Content-Type:";
        let result = Headers::parse_header_line(line);
        assert!(matches!(result, Err(HeaderError::EmptyHeaderValue)));
    }

    #[test]
    fn test_case_insensitive_headers() {
        let lines = "Content-Type: text/html\r\n\r\n";
        let mut headers = Headers::new();
        headers.parse_headers(lines).unwrap();

        assert_eq!(headers.get("content-type"), Some("text/html"));
        assert_eq!(headers.get("Content-Type"), Some("text/html"));
        assert_eq!(headers.get("CONTENT-TYPE"), Some("text/html"));
    }

    #[test]
    fn test_valid_special_characters_in_value() {
        let line = "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64)";
        let result = Headers::parse_header_line(line);
        assert!(result.is_ok());
    }

    #[test]
    fn test_insert_duplicate_headers() {
        let mut headers = Headers::new();
        headers.insert("Set-Cookie".to_string(), "session=abc".to_string());
        headers.insert("Set-Cookie".to_string(), "user=john".to_string());

        assert_eq!(headers.get("Set-Cookie"), Some("session=abc,user=john"));
    }
}
