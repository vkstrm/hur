use std::path::PathBuf;

use url::Url;

use crate::error;
use crate::error::Error;
use crate::http::headers::{Header, Headers};
use crate::io::read_file;

pub fn parse_url(url: &str) -> Result<Url, Error> {
    let parsed_url = match Url::parse(url) {
        Ok(url) => url,
        Err(why) => error!(&why.to_string()),
    };
    if !parsed_url.has_host() {
        error!("no host in input");
    }
    Ok(parsed_url)
}

pub struct InputBody {
    pub content: String,
    pub content_type: Option<String>,
}

pub fn parse_body(
    input_body: Option<String>,
    input_json: Option<String>,
    input_body_file: Option<PathBuf>,
) -> Result<Option<InputBody>, Error> {
    if let Some(body) = input_body {
        return Ok(Some(InputBody {
            content: body.to_string(),
            content_type: None,
        }));
    }
    if let Some(body) = input_json {
        match serde_json::from_str::<serde_json::Value>(&body) {
            Ok(_) => {
                return Ok(Some(InputBody {
                    content: body.to_string(),
                    content_type: Some("application/json".to_string()),
                }));
            }
            Err(why) => error!(&why.to_string()),
        };
    }
    if let Some(path) = input_body_file {
        let file_str = read_file(&path)?;
        let content_type = match path.extension() {
            Some(extension) if extension == "json" => Some("application/json".to_string()),
            _ => None,
        };
        return Ok(Some(InputBody {
            content: file_str,
            content_type,
        }));
    }

    Ok(None)
}

pub fn parse_headers(
    input_header: Option<Vec<Header>>,
    input_headers: Option<String>,
) -> Result<Headers, Error> {
    let mut headers = match input_header {
        Some(headers) => Headers::from(headers),
        None => Headers::new(),
    };

    if let Some(h) = input_headers {
        let h = json_headers(&h)?;
        headers.append(h)
    };

    Ok(headers)
}

fn json_headers(headers_json: &str) -> Result<Headers, Error> {
    let json_path = std::path::PathBuf::from(headers_json);
    let json_string = match json_path.extension() {
        Some(extension) if extension == "json" => read_file(&json_path)?,
        _ => headers_json.to_string(),
    };
    let map: std::collections::HashMap<String, String> = serde_json::from_str(&json_string)?;
    Ok(Headers::from(map))
}
