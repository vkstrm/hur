mod error;
mod http;
mod connector;
mod inout;
mod proxy;

use error::Error;
use http::request::{self, Request};

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
    let response = match proxy::should_proxy(&request)? {
        Some(addrs) => {
            match request.scheme {
                http::Scheme::HTTP => connector::http_request(addrs[0], &request.build_http_proxy())?,
                http::Scheme::HTTPS => {
                    let domain = request.domain.as_ref().unwrap().clone();
                    connector::proxy_https_request(addrs[0], &domain, &request.build())?
                },
            }
        },
        None => {
            let domain = request.domain.as_ref().ok_or("No domain").unwrap().clone();
            match request.scheme {
                http::Scheme::HTTP => connector::http_request(request.servers[0], &request.build())?,
                http::Scheme::HTTPS => connector::https_request(request.servers[0], &domain,&request.build())?, 
            }
        }
    };

    inout::output::handle_output(&response, &request, inout.output)
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