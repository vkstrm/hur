mod error;
mod http;
mod cli;
mod proxy;
mod logs;
mod requester;

use cli::output::{Output, handle_output};
use cli::Input;
use error::Error;
use http::{request::Request, response::Response};
use url::Url;

pub fn process(args: Vec<String>) {
    match handle_arguments(args) {
        Ok(()) => {},
        Err(why) => eprintln!("{}", why.to_string())
    }
}

fn handle_arguments(args: Vec<String>) -> Result<(), Error> {
    let (input, output) = cli::parse_args(args)?;
    handle_input(input, output)
}

fn handle_input(input: Input, output: Output) -> Result<(),Error> {
    let no_proxy = input.no_proxy;
    let request = setup_request(input)?;
    let request_output = serde_json::to_value(&request)?;

    let response = match no_proxy {
        false => request_with_proxy(request)?,
        true => no_proxy_request(request)?
    };

    handle_output(response, request_output, output)
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
    match input.body {
        Some(body) => Request::with_body(parsed_url, input.method, input.headers, &body),
        None => Request::new(parsed_url, input.method, input.headers)
    }
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