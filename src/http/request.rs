use std::convert::TryFrom;
use std::net::{SocketAddr, ToSocketAddrs};

use super::headers::Headers;
use super::{Method, Scheme};
use serde::Serialize;
use url::Url;

use crate::error::Error;
use crate::proxy::should_proxy;

#[derive(Serialize)]
pub struct Request {
    #[serde(skip)]
    pub url: Url,
    #[serde(skip)]
    pub proxy: bool,
    #[serde(skip)]
    pub servers: Vec<SocketAddr>,
    #[serde(rename = "url")]
    pub full_path: String,
    pub scheme: Scheme,
    pub protocol: String,
    pub method: Method,
    path: String,
    pub headers: Headers,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query: Option<String>,
}

impl Request {
    pub fn new(url: Url, method: Method, headers: Headers) -> Result<Request, Error> {
        let scheme = Scheme::try_from(url.scheme())?;
        let url_servers = find_socket_addresses(&url, &scheme)?;
        let (servers, proxy) = match should_proxy(&url, &url_servers, &scheme)? {
            Some(servers) => (servers, true),
            None => (url_servers, false),
        };

        Ok(Request {
            proxy,
            servers,
            full_path: url.to_string(),
            scheme,
            protocol: String::from("HTTP/1.1"),
            method,
            path: String::from(url.path()),
            headers: standard_headers(headers, &url.host().unwrap().to_string()),
            body: None,
            query: url.query().map_or_else(|| None, |s| Some(String::from(s))),
            url,
        })
    }

    pub fn with_body(
        url: Url,
        method: Method,
        headers: Headers,
        body: &str,
    ) -> Result<Request, Error> {
        let mut request = Request::new(url, method, headers)?;
        request.body = Some(body.to_string());
        request
            .headers
            .add("Content-Length", &body.as_bytes().len().to_string());
        Ok(request)
    }

    fn build_request(&self, path: &str) -> String {
        let mut message = self.make_status_line(path);
        self.add_headers(&mut message);
        self.add_body(&mut message);
        message.push_str("\r\n\r\n");
        message
    }

    pub fn build(&self) -> String {
        let path = match (self.proxy, &self.scheme) {
            (true, Scheme::Http) => &self.full_path,
            _ => &self.path,
        };
        self.build_request(path)
    }

    fn make_status_line(&self, path: &str) -> String {
        let path = match &self.query {
            Some(query) => format!("{path}?{query}"),
            None => path.to_string(),
        };
        format!(
            "{method} {path} {protocol}\r\n",
            method = self.method.to_string().to_uppercase(),
            path = path,
            protocol = self.protocol,
        )
    }

    fn add_body(&self, message: &mut String) {
        if let Some(body) = &self.body {
            message.push_str("\r\n");
            message.push_str(body);
        }
    }

    fn add_headers(&self, message: &mut String) {
        for (key, value_vec) in self.headers.iter() {
            for val in value_vec {
                message.push_str(&format!(
                    "{key}: {value}\r\n",
                    key = key,
                    value = val.trim(),
                ));
            }
        }
    }
}

fn find_socket_addresses(url: &Url, scheme: &Scheme) -> Result<Vec<SocketAddr>, Error> {
    let mut server_details = String::new();
    match url.domain() {
        Some(domain) => server_details.push_str(domain),
        None => server_details.push_str(&url.host().unwrap().to_string()),
    };
    server_details.push(':');
    match url.port() {
        Some(port) => server_details.push_str(&port.to_string()),
        None => match scheme {
            Scheme::Https => server_details.push_str("443"),
            Scheme::Http => server_details.push_str("80"),
        },
    }
    Ok(server_details.to_socket_addrs()?.collect())
}

fn standard_headers(input_headers: Headers, host: &str) -> Headers {
    let mut hs = Headers::new();
    hs.add(
        "User-Agent",
        &format!("{}/{}", clap::crate_name!(), clap::crate_version!()),
    );
    hs.add("Host", host);
    hs.add("Connection", "close");
    hs.append(input_headers);
    hs
}
