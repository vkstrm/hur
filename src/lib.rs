mod error;
mod http;
mod inout;
mod proxy;
mod logs;
mod requester;

use error::Error;
use http::{request::Request, headers::Headers, response::Response};
use http::urldetails::UrlDetails;
use url::Url;
use inout::{InputOutput, Input, output::handle_output};

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

fn handle_input(inout: InputOutput) -> Result<(),Error> {
    let no_proxy = inout.input.no_proxy;
    let request = setup_request(inout.input)?;
    let request_output = serde_json::to_value(&request)?;

    let response = match no_proxy {
        false => request_with_proxy(request)?,
        true => no_proxy_request(request)?
    };

    handle_output(response, request_output, inout.output)
}

fn request_with_proxy(request: Request) -> Result<Response, Error> {
    match proxy::should_proxy(&request)? {
        Some(addrs) => requester::send_proxy_request(request, addrs),
        None => requester::send_request(request)
    }
}

fn no_proxy_request(request: Request) -> Result<Response, Error> {
    requester::send_request(request)
}

fn setup_request(input: Input) -> Result<Request, Error> {
    let parsed_url = parse_url(&input.url)?;
    let url_details = UrlDetails::from_url(&parsed_url)?;
    let headers = standard_headers(input.headers, &url_details.host);
    match input.body {
        Some(body) => Request::with_body(input.method, url_details, headers, &body),
        None => Request::new(input.method, url_details, headers)
    }
}

fn standard_headers(input_headers: Headers, host: &str) -> Headers {
    let mut hs = Headers::new();
    hs.add("User-Agent", &format!("{}/{}", clap::crate_name!(), clap::crate_version!()));
    hs.add("Host", host);
    hs.add("Connection", "close");
    hs.append(input_headers);
    hs
}

fn parse_url(url: &str) -> Result<Url, Error> {
    let parsed_url = match Url::parse(url) {
        Ok(url) => url,
        Err(why) => error!(&why.to_string())
    };
    if !parsed_url.has_host() {
        error!("no host in input");
    }
    Ok(parsed_url)
}