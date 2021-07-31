use std::collections::HashMap;

#[derive(Debug)]
pub enum Method {
    GET,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct HttpRequest {
    pub method: Method,
    path: String,
    headers: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new(method: Method, url: &url::Url) -> HttpRequest {
        let mut headers = HashMap::<String, String>::new();
        headers.insert("Host".to_string(), String::from(url.host().unwrap().to_string()));
        headers.insert("Connection".to_string(), "close".to_string());
        HttpRequest{
            method,
            path: String::from(url.path()),
            headers,
        }
    }

    pub fn build(&self) -> String {
        let mut message = format!(
            "{method} {path} HTTP/1.1\r\n",
            method = self.method.to_string(),
            path = self.path,
        );

        for (key, value) in &self.headers {
            message.push_str(
                &format!(
                    "{key}: {value}\r\n",
                    key = key,
                    value = value,
                )
            );
        }

        // Depending on method, add body and so on

        message.push_str("\r\n\r\n");        
        message
    }

    pub fn set_header(&mut self, key: &str, value: &str) {
        self.headers.insert(String::from(key), String::from(value));
    }
}