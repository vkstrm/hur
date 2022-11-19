mod cli;
mod error;
mod http;
mod logs;
mod proxy;
mod requester;

use cli::output::{handle_output, Output};
use cli::Input;
use error::Error;
use http::request::Request;
use requester::connector::{Connector, ProxyConnector, RegularConnector};
use requester::Requester;
use url::Url;

pub fn process(args: Vec<String>) {
    match handle_arguments(args) {
        Ok(()) => {}
        Err(error) => eprintln!("{}", error),
    }
}

fn handle_arguments(args: Vec<String>) -> Result<(), Error> {
    let (input, output) = cli::parse_args(args)?;
    handle_input(input, output)
}

fn handle_input(input: Input, output: Output) -> Result<(), Error> {
    let allow_proxy = input.allow_proxy;
    let timeout = input.timeout;
    let request = setup_request(input)?;

    let connector: Box<dyn Connector> = match (request.proxy, allow_proxy) {
        (true, true) => Box::new(ProxyConnector::new(timeout)),
        _ => Box::new(RegularConnector::new(timeout)),
    };

    let requester = Requester::new(connector);

    let request_output = serde_json::to_value(&request)?;
    let response = requester.send_request(request)?;

    handle_output(response, request_output, output)
}

fn setup_request(input: Input) -> Result<Request, Error> {
    let parsed_url = parse_url(&input.url)?;
    match input.body {
        Some(body) => Request::with_body(parsed_url, input.method, input.headers, &body),
        None => Request::new(parsed_url, input.method, input.headers),
    }
}

fn parse_url(url: &str) -> Result<Url, Error> {
    let parsed_url = match Url::parse(url) {
        Ok(url) => url,
        Err(why) => error!(&why.to_string()),
    };
    if !parsed_url.has_host() {
        error!("no host in input");
    }
    Ok(parsed_url)
}
