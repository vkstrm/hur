use crate::error;
use crate::error::Error;
use std::convert::TryFrom;

pub mod headers;
pub mod request;
pub mod response;

#[derive(serde::Serialize, Debug, Clone, clap::ValueEnum)]
#[serde(rename_all = "UPPERCASE")]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Trace,
    Patch,
    Connect,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum Scheme {
    Http,
    Https,
}

impl TryFrom<&str> for Scheme {
    type Error = Error;

    fn try_from(s: &str) -> Result<Scheme, Self::Error> {
        match s {
            "http" => Ok(Scheme::Http),
            "https" => Ok(Scheme::Https),
            _ => error!("hur only supports http/s"),
        }
    }
}
