use std::collections::HashMap;

use super::error::Error;

#[derive(Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Request {
    pub method: Method,
    path: String,
    pub headers: Headers,
    body: Option<String>,
}

impl Request {
    pub fn new(method: Method, url: &url::Url) -> Request {
        let mut headers = Headers::new();
        headers.add("Host", &url.host().unwrap().to_string());
        headers.add("Connection", "close");
        Request{
            method,
            path: String::from(url.path()),
            headers,
            body: None,
        }
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

#[derive(Debug)]
pub struct Response {
    pub protocol: String,
    pub status_code: i32,
    pub reason_phrase: String,
    pub headers: Headers,
    pub body: Option<String>,
}

impl Response {
    pub fn from_response(response: &[u8]) -> Result<Response, Error> {
        let response_string = String::from_utf8_lossy(response);
        let mut lines = response_string.lines();
        let status_line = lines.next();
        if status_line.is_none() { 
            return Err(Error::new("no status line in response"))
        }
        let splits: Vec<&str> = status_line.unwrap().splitn(3, ' ').collect();
        let protocol = splits[0].to_string();
        let status_code = splits[1].parse::<i32>().unwrap();
        let reason_phrase = splits[2].to_string();

        // Collect headers
        let mut headers = Headers::new();
        loop {
            let line = lines.next();
            if line.is_none() || line.unwrap() == "" {
                break;
            }
            let splits: Vec<&str> = line.unwrap().splitn(2, ':').collect();
            headers.add(splits[0], splits[1].trim());
        }

        // Collect body
        let mut body_string = String::new();
        for line in lines {
            body_string.push_str(&format!("{line}\n",line = line));
        }
        let body = match body_string.is_empty() {
            true => None,
            false => Some(body_string),
        };

        Ok(Response {
            protocol,
            status_code,
            reason_phrase,
            headers,
            body,
        })
    }
}

#[derive(Debug)]
pub struct Headers {
    pub headers_map: HashMap<String, Vec<String>>
}

impl Headers {
    pub fn new() -> Headers {
        Headers {
            headers_map: HashMap::<String, Vec<String>>::new()
        }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        if self.headers_map.contains_key(key) {
            match self.headers_map.get_mut(key) {
                Some(vec) => vec.push(value.to_string()),
                None => panic!("no vec for key"),
            }
        } else {
            self.headers_map.insert(key.to_string(), vec![value.to_string()]);
        }
    }

    pub fn append(&mut self, other: Headers) {
        for (key, val) in other.headers_map {
            let key = capitalize(&key);
            match key.as_str() {
                "Connection" | "connection" | "Host" | "host" => {
                    self.headers_map.insert(key, val);
                },
                _ => {
                    for v in val {
                        self.add(&key, &v);
                    }
                }
            }
        }
    }
}

fn capitalize(string: &String) -> String {
    let mut chars = string.chars();
    return chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str();
} 