use thiserror::Error;

#[derive(Debug, Error)]
pub enum BodyError {
    #[error("Invalid content length: {0}")]
    InvalidContentLength(String),

    #[error("Unexpected end of body: expected {expected} bytes, got {actual}")]
    UnexpectedEof { expected: usize, actual: usize },

    #[error("Missing Content-Length header for request with body")]
    MissingContentLength,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Body {
    Empty,
    Content(Vec<u8>),
}

impl Body {
    pub fn new() -> Self {
        Body::Empty
    }

    pub fn from_content_length(buf: &[u8], length: usize) -> Result<Self, BodyError> {
        if length == 0 {
            return Ok(Body::Empty);
        }

        if buf.len() < length {
            return Err(BodyError::UnexpectedEof {
                expected: length,
                actual: buf.len(),
            });
        }

        let body = buf[..length].to_vec();
        Ok(Body::Content(body))
    }

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Body::Empty => &[],
            Body::Content(data) => data.as_slice(),
        }
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.as_bytes())
    }

    pub fn len(&self) -> usize {
        match self {
            Body::Empty => 0,
            Body::Content(data) => data.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Body::Empty)
    }
}

impl Default for Body {
    fn default() -> Self {
        Self::new()
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Body::Content(s.into_bytes())
    }
}

impl From<&str> for Body {
    fn from(s: &str) -> Self {
        Body::Content(s.as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_body() {
        let body = Body::from_content_length(&[], 0).unwrap();
        assert_eq!(body, Body::Empty);
        assert!(body.is_empty());
        assert_eq!(body.len(), 0);
    }

    #[test]
    fn test_body_with_content() {
        let data = b"Hello, World!";
        let body = Body::from_content_length(data, data.len()).unwrap();

        assert_eq!(body.as_bytes(), b"Hello, World!");
        assert_eq!(body.as_str().unwrap(), "Hello, World!");
        assert_eq!(body.len(), 13);
        assert!(!body.is_empty());
    }

    #[test]
    fn test_unexpected_eof() {
        let data = b"Hello";
        let result = Body::from_content_length(data, 10);

        assert!(matches!(
            result,
            Err(BodyError::UnexpectedEof {
                expected: 10,
                actual: 5
            })
        ));
    }

    #[test]
    fn test_body_as_str_invalid_utf8() {
        let data = vec![0xFF, 0xFE, 0xFD];
        let body = Body::Content(data);

        assert!(body.as_str().is_err());
    }
}
