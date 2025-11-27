use std::collections::HashMap;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Invalid query format string")]
    InvalidFormat,
    #[error("Invalid URL encoding")]
    InvalidEncoding,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    params: HashMap<String, Vec<String>>,
}

impl Query {
    pub fn new() -> Self {
        Query {
            params: HashMap::new(),
        }
    }

    pub fn parse(query_string: &str) -> Result<Self, QueryError> {
        let mut query = Query::new();

        if query_string.is_empty() {
            return Ok(query);
        }

        for pair in query_string.split('&') {
            if pair.is_empty() {
                continue;
            }

            let (key, value) = if let Some(pos) = pair.find('=') {
                let key = &pair[..pos];
                let value = &pair[pos + 1..];
                (key, value)
            } else {
                (pair, "")
            };

            let key = Self::decode_url(key)?;
            let value = Self::decode_url(value)?;

            query.params.entry(key).or_default().push(value);
        }

        Ok(query)
    }

    pub fn from_url(url: &str) -> Result<Self, QueryError> {
        if let Some(pos) = url.find('?') {
            let query_string = &url[pos + 1..];
            Self::parse(query_string)
        } else {
            Ok(Query::new())
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.params.get(key)?.first().map(|s| s.as_str())
    }

    pub fn get_all(&self, key: &str) -> Option<&[String]> {
        self.params.get(key).map(|v| v.as_slice())
    }

    pub fn contains(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.params.len()
    }

    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.params
            .iter()
            .filter_map(|(k, v)| v.first().map(|val| (k.as_str(), val.as_str())))
    }

    pub fn iter_all(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.params
            .iter()
            .flat_map(|(k, values)| values.iter().map(move |v| (k.as_str(), v.as_str())))
    }

    pub fn decode_url(s: &str) -> Result<String, QueryError> {
        let mut bytes = Vec::with_capacity(s.len());
        let mut chars = s.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '%' => {
                    let hex: String = chars.by_ref().take(2).collect();
                    if hex.len() != 2 {
                        return Err(QueryError::InvalidEncoding);
                    }
                    let byte =
                        u8::from_str_radix(&hex, 16).map_err(|_| QueryError::InvalidEncoding)?;
                    bytes.push(byte);
                }
                '+' => {
                    bytes.push(b' ');
                }
                _ => {
                    let mut buf = [0; 4];
                    let encoded = ch.encode_utf8(&mut buf);
                    bytes.extend_from_slice(encoded.as_bytes());
                }
            }
        }

        String::from_utf8(bytes).map_err(|_| QueryError::InvalidEncoding)
    }

    pub fn encode_url(s: &str) -> String {
        let mut encoded = String::new();

        for byte in s.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    encoded.push(byte as char);
                }
                b' ' => {
                    encoded.push('+');
                }
                _ => {
                    encoded.push_str(&format!("%{:02X}", byte));
                }
            }
        }

        encoded
    }
}

impl Default for Query {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let q = Query::parse("key=value").unwrap();
        assert_eq!(q.get("key"), Some("value"));
    }

    #[test]
    fn test_parse_multiple() {
        let q = Query::parse("key=value&key2=value2").unwrap();
        assert_eq!(q.get("key"), Some("value"));
        assert_eq!(q.get("key2"), Some("value2"));
    }

    #[test]
    fn test_parse_encoded() {
        let q = Query::parse("key=hello%20world").unwrap();
        assert_eq!(q.get("key"), Some("hello world"));
    }

    #[test]
    fn test_parse_utf8() {
        // "key=café" -> café is usually encoded
        // café in UTF-8 is c3 a9
        let q = Query::parse("key=caf%C3%A9").unwrap();
        assert_eq!(q.get("key"), Some("café"));
    }

    #[test]
    fn test_parse_plus_as_space() {
        let q = Query::parse("key=hello+world").unwrap();
        assert_eq!(q.get("key"), Some("hello world"));
    }

    #[test]
    fn test_parse_empty_value() {
        let q = Query::parse("key=").unwrap();
        assert_eq!(q.get("key"), Some(""));
    }

    #[test]
    fn test_parse_no_value() {
        let q = Query::parse("key").unwrap();
        assert_eq!(q.get("key"), Some(""));
    }

    #[test]
    fn test_parse_multiple_values_for_key() {
        let q = Query::parse("key=1&key=2").unwrap();
        let values = q.get_all("key").unwrap();
        assert_eq!(values, vec!["1", "2"]);
    }

    #[test]
    fn test_encode_url() {
        assert_eq!(Query::encode_url("hello world"), "hello+world");
        assert_eq!(Query::encode_url("café"), "caf%C3%A9");
        assert_eq!(Query::encode_url("a/b?c=d"), "a%2Fb%3Fc%3Dd");
    }
}
