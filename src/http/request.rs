use std::net::{SocketAddr, ToSocketAddrs};

use super::Method;
use super::headers::Headers;
use url::Url;
use serde::Serialize;

use crate::error::Error;

#[derive(Serialize)]
pub struct Request {
    pub method: Method,
    path: String,
    pub headers: Headers,
    body: Option<String>,
    pub domain: Option<String>,
    pub scheme: String,
    pub servers: Vec<SocketAddr>
}

impl Request {
    pub fn new(method: Method, url: &str) -> Result<Request, Error> {
        let parsed_url = parse_url(url)?;
        let url_details = UrlDetails::from_url(&parsed_url);
        let servers = url_details.find_socket_addresses()?;
        let mut headers = Headers::new();
        headers.add("Host", &url_details.host);
        headers.add("Connection", "close");
        Ok(Request{
            method,
            path: url_details.path,
            headers,
            body: None,
            domain: url_details.domain,
            scheme: url_details.scheme,
            servers,
        })
    }

    pub fn build(&self) -> String {
        let mut message = format!(
            "{method} {path} HTTP/1.1\r\n",
            method = self.method.to_string(),
            path = self.path,
        );

        // Add headers
        for (key, value_vec) in &self.headers.headers_map {
            for val in value_vec {
                message.push_str(
                    &format!(
                        "{key}: {value}\r\n",
                        key = key,
                        value = val.trim(),
                    )
                );
            }
        }

        // Add body
        if let Some(body) = &self.body {
            message.push_str("\r\n");
            message.push_str(body);
        }

        // Done
        message.push_str("\r\n\r\n");        
        message
    }

    pub fn set_body(&mut self, body: &str) {
        self.body = Some(body.to_string());
    }
}

fn parse_url(url: &str) -> Result<url::Url, Error> {
    let parsed_url = match Url::parse(url) {
        Ok(url) => url,
        Err(why) => return Err(Error::new(&why.to_string()))
    };
    if !parsed_url.has_host() {
        return Err(Error::new("no host in input"))
    }
    Ok(parsed_url)
}

struct UrlDetails {
    path: String,
    domain: Option<String>,
    port: Option<u16>,
    host: String,
    scheme: String,
}

impl UrlDetails {
    pub fn from_url(url: &url::Url) -> UrlDetails {
        UrlDetails {
            path: url.path().to_string(),
            domain: match url.domain() {
                Some(domain) => Some(domain.to_string()),
                None => None,
            },
            port: url.port(),
            host: match url.host_str() {
                Some(host) => host.to_string(),
                None => String::new(),
            },
            scheme: url.scheme().to_string()
        }
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
                match self.scheme.as_str() {
                    "https" => server_details.push_str("443"),
                    "http" => server_details.push_str("80"),
                    _ => return Err(Error::new("only support http/s"))
                }
            }
        }
        Ok(server_details.to_socket_addrs()?.collect())
    }
}