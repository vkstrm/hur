use super::http::{response::Response, headers::Headers};
use crate::error::Error;

pub struct Output {
    pub verbose: bool,
    pub query_header: Option<String>,
    pub no_body: bool,
}

pub fn handle_output(mut response: Response, request: serde_json::Value, output: Output) -> Result<(), Error> {
    if output.verbose {
        if output.no_body {
            response.body = Some(String::from("Gulp, I swallowed the body!")); 
        }
        let json_output = serde_json::json!({"request": request, "response":response});
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else if let Some(h) = output.query_header {
        query_header(&h, &response.headers)
    } else {
        match &response.body {
            Some(body) => if !output.no_body { println!("{}", body) },
            None => {},
        }
    }
    Ok(())
}

fn query_header(header: &str, headers: &Headers) {
    let h = header.to_lowercase();
    for (key, value) in &headers.headers_map {
        if h == key.to_lowercase() {
            for val in value {
                println!("{}", val);
            }
        }
    }
}