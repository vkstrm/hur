use std::net::{SocketAddr, ToSocketAddrs};

use super::{Method, Scheme};
use super::headers::Headers;
use url::Url;
use serde::Serialize;

use crate::error::Error;

#[derive(Serialize)]
pub struct Request {
    pub protocol: String,
    pub method: Method,
    path: String,
    pub full_path: String,
    pub headers: Headers,
    body: Option<String>,
    pub domain: Option<String>,
    pub scheme: Scheme,
    pub servers: Vec<SocketAddr>,
    pub host: String,
}

impl Request {
    pub fn new(method: Method, url: &str, headers: Option<Headers>) -> Result<Request, Error> {
        let parsed_url = parse_url(url)?;
        let url_details = UrlDetails::from_url(&parsed_url)?;
        let servers = url_details.find_socket_addresses()?;
        let mut hs = Headers::new();

        hs.add("Host", &format!("{0}", url_details.host.as_str()));
        hs.add("Connection", "close");
        if let Some(headers) = headers {
            hs.append(headers);
        }

        Ok(Request{
            protocol: String::from("HTTP/1.1"),
            method,
            path: url_details.path,
            full_path: parsed_url.to_string(),
            headers: hs,
            body: None,
            domain: url_details.domain,
            scheme: url_details.scheme,
            servers,
            host: url_details.host,
        })
    }

    pub fn with_body(method: Method, url: &str, headers: Option<Headers>, body: &str) -> Result<Request, Error> {
        let mut request = Request::new(method, url, headers)?;
        request.body = Some(body.to_string());
        Ok(request)
    }

    pub fn with_json(method: Method, url: &str, headers: Option<Headers>, body: &str) -> Result<Request, Error> {
        let mut request = Request::with_body(method, url, headers, body)?;
        request.headers.add("Content-Type", "application/json");
        Ok(request)
    }

    fn build_request(&self, path: &str) -> String {
        let mut message = format!(
            "{method} {path} {protocol}\r\n",
            method = self.method.to_string(),
            path = path,
            protocol = self.protocol,
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

    pub fn build(&self) -> String {
        self.build_request(&self.path)
    }

    pub fn build_http_proxy(&self) -> String {
        self.build_request(&self.full_path)
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
    scheme: Scheme,
}

impl UrlDetails {
    pub fn from_url(url: &url::Url) -> Result<UrlDetails, Error> {
        Ok(UrlDetails {
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
            scheme: match url.scheme() {
                "http" => Scheme::HTTP,
                "https" => Scheme::HTTPS,
                _ => return Err(Error::new("only support http/s"))
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