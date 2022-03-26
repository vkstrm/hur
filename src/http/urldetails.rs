use std::net::{SocketAddr, ToSocketAddrs};

use crate::error::Error;
use crate::error;
use super::Scheme;

pub struct UrlDetails {
    pub path: String,
    pub full_path: String,
    pub domain: Option<String>,
    pub port: Option<u16>,
    pub scheme: Scheme,
    pub host: String,
}

impl UrlDetails {
    pub fn from_url(url: &url::Url) -> Result<UrlDetails, Error> {
        Ok(UrlDetails {
            path: url.path().to_string(),
            full_path: url.to_string(),
            domain: url.domain().map(|domain| domain.to_string()),
            port: url.port(),
            host: match url.host_str() {
                Some(host) => host.to_string(),
                None => String::new(),
            },
            scheme: match url.scheme() {
                "http" => Scheme::HTTP,
                "https" => Scheme::HTTPS,
                _ => error!("only support http/s")
            }
        })
    }

    pub fn find_socket_addresses(&self) -> Result<Vec<SocketAddr>, Error> {
        let mut server_details = String::new();
        match &self.domain {
            Some(domain) => server_details.push_str(domain.as_str()),
            None => server_details.push_str(&self.host), 
        };
        server_details.push(':');
        match self.port {
            Some(port) => server_details.push_str(&port.to_string()),
            None => {
                match self.scheme {
                    Scheme::HTTPS => server_details.push_str("443"),
                    Scheme::HTTP => server_details.push_str("80")
                }
            }
        }
        Ok(server_details.to_socket_addrs()?.collect())
    }
}