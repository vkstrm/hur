use super::http;
use crate::error::Error;

pub struct Output {
    pub verbose: bool,
    pub query_header: Option<String>
}

#[derive(serde::Serialize)]
struct OutputJson<'a> {
    request: &'a http::request::Request,
    response: &'a http::response::Response
}

pub fn handle_output(response: &http::response::Response, request: &http::request::Request, output: Output) -> Result<(), Error> {
    if output.verbose {
        print_verbose(response, request)?;
    } else if let Some(h) = output.query_header {
        query_header(&h, &response.headers)
    } else {
        match &response.body {
            Some(body) => println!("{}", body),
            None => {},
        }
    }
    Ok(())
}

fn print_verbose(response: &http::response::Response, request: &http::request::Request) -> Result<(), Error>{
    let output_json = OutputJson {
        request,
        response
    };
    let json = serde_json::to_string_pretty(&output_json)?;
    println!("{}", json);
    Ok(())
}

fn query_header(header: &str, headers: &http::headers::Headers) {
    let h = header.to_lowercase();
    for (key, value) in &headers.headers_map {
        if h == key.to_lowercase() {
            for val in value {
                println!("{}", val);
            }
        }
    }
}