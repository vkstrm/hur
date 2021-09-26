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
    perform(inout)
}

fn perform(inout: inout::InOut) -> Result<(),Error> {
    let request = setup_request(inout.input)?;
    let response = send_request(&request)?;
    let output = OutputJson {
        request: &request,
        response: &response
    };

    if inout.output.verbose {
        let json = serde_json::to_string_pretty(&output).unwrap();
        println!("{}", json);
    } else {
        match response.body {
            Some(body) => println!("{}", body),
            None => {},
        }
    }

    Ok(())
}

fn setup_request(input: inout::Input) -> Result<request::Request, Error> {
    let mut request = Request::new(input.method, input.url.as_str())?;
    if let Some(headers) = input.headers {
        request.headers.append(headers);
    }
    if let Some(body) = &input.body {
        request.set_body(body);
        if input.json {
            request.headers.add("Content-Type", "application/json")
        }
    }

    Ok(request)
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