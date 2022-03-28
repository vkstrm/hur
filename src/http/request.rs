use std::convert::TryFrom;
use std::net::{ToSocketAddrs, SocketAddr};

use super::{Method, Scheme};
use super::headers::Headers;
use serde::Serialize;
use url::Url;

use crate::error::Error;

#[derive(Serialize)]
pub struct Request {
    #[serde(skip)]
    pub url: Url,
    #[serde(rename = "url")]
    pub full_path: String,
    pub scheme: Scheme,
    pub protocol: String,
    pub method: Method,
    path: String,
    pub headers: Headers,
    body: Option<String>,
}

impl Request {
    pub fn new(url: Url, method: Method, headers: Headers) -> Result<Request, Error> {
        Ok(Request{
            full_path: url.to_string(),
            scheme: Scheme::try_from(url.scheme())?,
            protocol: String::from("HTTP/1.1"),
            method,
            path: String::from(url.path()),
            headers: standard_headers(headers, &url.host().unwrap().to_string()), // TODO Gör bättre 
            body: None,
            url,
        })
    }

    pub fn with_body(url: Url, method: Method, headers: Headers, body: &str) -> Result<Request, Error> {
        let mut request = Request::new(url, method, headers)?;
        request.body = Some(body.to_string());
        Ok(request)
    }

    pub fn find_socket_addresses(&self) -> Result<Vec<SocketAddr>, Error> {
        let mut server_details = String::new();
        match &self.url.domain() {
            Some(domain) => server_details.push_str(domain),
            None => server_details.push_str(&self.url.host().unwrap().to_string()), 
        };
        server_details.push(':');
        match self.url.port() {
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

fn standard_headers(input_headers: Headers, host: &str) -> Headers {
    let mut hs = Headers::new();
    hs.add("User-Agent", &format!("{}/{}", clap::crate_name!(), clap::crate_version!()));
    hs.add("Host", host);
    hs.add("Connection", "close");
    hs.append(input_headers);
    hs
}