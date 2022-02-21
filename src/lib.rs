use std::net::SocketAddr;

mod error;
mod http;
mod inout;
mod proxy;
mod logs;
mod requester;

use error::Error;
use http::{Scheme, UrlDetails, request::Request, headers::Headers, response::Response};
use url::Url;
use inout::{InOut, Input, output::handle_output};

pub fn process(args: Vec<String>) {
    match handle_arguments(args) {
        Ok(()) => {},
        Err(why) => eprintln!("{}", why.to_string())
    }
}

fn handle_arguments(args: Vec<String>) -> Result<(), Error> {
    let inout = inout::parse_args(args)?;
    handle_input(inout)
}

fn handle_input(inout: InOut) -> Result<(),Error> {
    let parsed_url = parse_url(&inout.input.url)?;
    let url_details = UrlDetails::from_url(&parsed_url)?;

    let request = setup_request(inout.input, url_details)?;
    let request_output = serde_json::to_value(&request)?;

    let response = match proxy::should_proxy(&request)? {
        Some(proxy_addrs) => send_proxy_request(request, proxy_addrs)?,
        None => send_request(request)?,
    };

    handle_output(response, request_output, inout.output)
}

fn send_proxy_request(request: Request, addrs: Vec<SocketAddr>) -> Result<Response, Error> {
    match request.scheme {
        Scheme::HTTP => requester::proxy_http(request, addrs),
        Scheme::HTTPS => requester::proxy_https(request, addrs)
    }
}

fn send_request(request: Request) -> Result<Response, Error> {
    match request.scheme {
        Scheme::HTTP => requester::http(request),
        Scheme::HTTPS => requester::https(request)
    }
}

fn standard_headers(input_headers: Option<Headers>, host: &str) -> Headers {
    let mut hs = Headers::new();
    hs.add("User-Agent", &format!("{}/{}", clap::crate_name!(), clap::crate_version!()));
    hs.add("Host", host);
    hs.add("Connection", "close");
    if let Some(headers) = input_headers {
        hs.append(headers);
    }
    hs
}

fn setup_request(input: Input, url_details: UrlDetails) -> Result<Request, Error> {
    let mut headers = standard_headers(input.headers, &url_details.host);
    if input.json {
        headers.add("Content-Type", "application/json");
    }
    match input.body {
        Some(body) => Request::with_body(input.method, url_details, headers, &body),
        None => Request::new(input.method, url_details, headers)
    }
}

fn parse_url(url: &str) -> Result<Url, Error> {
    let parsed_url = match Url::parse(url) {
        Ok(url) => url,
        Err(why) => return Err(Error::new(&why.to_string()))
    };
    if !parsed_url.has_host() {
        return Err(Error::new("no host in input"))
    }
    Ok(parsed_url)
}