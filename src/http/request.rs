use std::net::SocketAddr;

use super::{Method, Scheme, urldetails::UrlDetails};
use super::headers::Headers;
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
    pub fn new(method: Method, url_details: UrlDetails, headers: Headers) -> Result<Request, Error> {
        let servers = url_details.find_socket_addresses()?;

        Ok(Request{
            protocol: String::from("HTTP/1.1"),
            method,
            path: url_details.path,
            full_path: url_details.full_path,
            headers,
            body: None,
            domain: url_details.domain,
            scheme: url_details.scheme,
            servers,
            host: url_details.host,
        })
    }

    pub fn with_body(method: Method, url: UrlDetails, headers: Headers, body: &str) -> Result<Request, Error> {
        let mut request = Request::new(method, url, headers)?;
        request.body = Some(body.to_string());
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