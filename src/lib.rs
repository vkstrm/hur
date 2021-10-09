use std::net::SocketAddr;

mod error;
mod http;
mod connector;
mod inout;

use error::Error;
use http::request::{self, Request};

#[derive(serde::Serialize)]
struct OutputJson<'a> {
    request: &'a http::request::Request,
    response: &'a http::response::Response
}

pub fn process(args: &Vec<String>) {
    match handle_arguments(&args) {
        Ok(()) => {},
        Err(why) => eprintln!("{}", why.to_string())
    }
}

fn handle_arguments(args: &Vec<String>) -> Result<(), Error> {
    let inout = inout::parse_args(args)?;
    handle_input(inout)
}

fn handle_input(inout: inout::InOut) -> Result<(),Error> {
    let request = setup_request(inout.input)?;
    let response = send_request(&request)?;
    let output = OutputJson {
        request: &request,
        response: &response
    };

    if inout.output.verbose {
        let json = serde_json::to_string_pretty(&output).unwrap();
        println!("{}", json);
    } else if let Some(h) = inout.output.query_header {
        query_header(&h, response.headers)
    } else {
        match response.body {
            Some(body) => println!("{}", body),
            None => {},
        }
    }

    Ok(())
}

fn query_header(header: &str, headers: http::headers::Headers) {
    let h = header.to_lowercase();
    for (key, value) in headers.headers_map {
        if h == key.to_lowercase() {
            for val in value {
                println!("{}", val);
            }
        }
    }
}

fn setup_request(input: inout::Input) -> Result<request::Request, Error> {
    match input.body {
        Some(body) => {
            match input.json {
                true => Request::with_json(input.method, &input.url, input.headers, &body),
                false => Request::with_body(input.method, &input.url, input.headers, &body),
            }
        },
        None => Request::new(input.method, &input.url, input.headers)
    }
}

fn send_request(request: &http::request::Request) -> Result<http::response::Response, Error> {
    let scheme = request.scheme.clone();
    for server in &request.servers {
        let request_result = match scheme.as_str() {
            "http" => send_http_request(*server, &request.build()),
            "https" => send_https_request(*server, &request.build(), &request.domain),
            _ => Err(Error::new("only http/s supported")),
        };
        if request_result.is_ok() {
            return Ok(request_result.unwrap())
        }
    }

    Err(Error::new("no server worked"))
}

fn send_http_request(server: SocketAddr, request_str: &str) -> Result<http::response::Response, Error> {
    let mut response_buffer = vec![];
    match connector::do_http_request(
            server,
            request_str.as_bytes(),
            &mut response_buffer) {
                Ok(()) => http::response::Response::from_response(&response_buffer),
                Err(why) => Err(why)
            }
}

fn send_https_request(server: SocketAddr, request_str: &str, domain: &Option<String>) -> Result<http::response::Response, Error> {
    let mut response_buffer = vec![];
    let domain = match domain {
        Some(domain) => domain,
        None => return Err(Error::new("Need domain for HTTPS"))
    };
    match connector::do_https_request(
        server, 
        domain,
        request_str.as_bytes(), 
        &mut response_buffer) {
            Ok(()) => match http::response::Response::from_response(&response_buffer) {
                    Ok(response) => Ok(response),
                    Err(why) => Err(why),
                },
            Err(why) => Err(why)
        }
}