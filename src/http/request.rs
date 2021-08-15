use super::Method;
use super::headers::Headers;
use url::Url;

use crate::error::Error;

pub struct Request {
    pub method: Method,
    path: String,
    pub headers: Headers,
    body: Option<String>,
    pub url: url::Url,
}

impl Request {
    pub fn new(method: Method, url: &str) -> Result<Request, Error> {
        let parsed_url = match Url::parse(url) {
            Ok(url) => url,
            Err(why) => return Err(Error::new(&why.to_string()))
        };
        if !parsed_url.has_host() {
            return Err(Error::new("no host in input"))
        }

        let mut headers = Headers::new();
        headers.add("Host", &parsed_url.host().unwrap().to_string());
        headers.add("Connection", "close");
        Ok(Request{
            method,
            path: String::from(parsed_url.path()),
            headers,
            body: None,
            url: parsed_url,
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

    pub fn set_body(&mut self, body: String) {
        self.body = Some(body);
    }
}

impl super::Printer for Request {
    fn print_headers(&self, verbose: bool) {
        if !verbose { return }
        println!("--- Request Headers ---");
        for (key, values) in &self.headers.headers_map {
            let joined = values.join(",");
            println!("{}: {}", key, joined);
        }
    }

    fn print_body(&self, verbose: bool) {
        if !verbose { return }
        match &self.body {
            Some(body) => {
                println!("--- Request Body ---");
                println!("{}", body);
            }
            None => {},
        }
    }
}