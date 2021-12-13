use super::headers::Headers;
use crate::error::Error;

#[derive(serde::Serialize, Debug)]
pub struct Response {
    pub protocol: String,
    pub status_code: i32,
    pub reason_phrase: String,
    pub headers: Headers,
    pub body: Option<String>,
    pub raw: Option<String>,
}

impl Response {
    pub fn from_response(response: &[u8]) -> Result<Response, Error> {
        let response_string = String::from_utf8_lossy(response);
        let raw = Some(response_string.clone().into_owned());
        let mut lines = response_string.lines();
        
        let status_line: &str;
        loop {
            let line = lines.next();
            if line.is_none() {
                return Err(Error::new("no status line in response"))
            }

            let line = line.unwrap();
            if line.is_empty() {
                continue;
            }

            status_line = line;
            break;
        }

        let splits: Vec<&str> = status_line.splitn(3, ' ').collect();
        if splits.len() != 3 {
            return Err(Error::new("Weird status line"))
        }

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

        // Collect body // TODO Ignore if request was HEAD
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
            raw,
        })
    }
}