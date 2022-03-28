use crate::error;
use crate::error::Error;
use std::convert::TryFrom;

pub mod response;
pub mod request;
pub mod headers;

#[derive(serde::Serialize, Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    TRACE,
    PATCH,
    CONNECT,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(serde::Serialize, Debug)]
pub enum Scheme {
    HTTP,
    HTTPS
}

impl TryFrom<&str> for Scheme {
    type Error = Error;

    fn try_from(s: &str) -> Result<Scheme, Self::Error> {
        match s {
            "http" => Ok(Scheme::HTTP),
            "https" => Ok(Scheme::HTTPS),
            _ => error!("hur only supports http/s")
        }
    }
}