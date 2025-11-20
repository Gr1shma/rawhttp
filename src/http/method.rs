use std::str::FromStr;

use crate::http::request::ParseError;

#[derive(Debug, Clone, PartialEq)]
pub enum Method {
    GET,
    PUT,
    POST,
    DELETE,
}

impl FromStr for Method {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            _ => Err(ParseError::InvalidMethod(s.to_string())),
        }
    }
}
