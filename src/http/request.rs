use super::Method;
use super::headers::Headers;

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