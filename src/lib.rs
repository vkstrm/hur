mod cli;
mod error;
mod http;
mod logs;
mod proxy;
mod requester;

use cli::output::handle_output;
use error::Error;
use http::request::Request;
use requester::Requester;
use requester::connector::{Connector, ProxyConnector, RegularConnector};

pub fn handle(args: Vec<String>) {
    match handle_args(args) {
        Ok(()) => {}
        Err(error) => eprintln!("{}", error),
    }
}

fn handle_args(args: Vec<String>) -> Result<(), Error> {
    let inputs = cli::parse_input(args)?;

    let request = match inputs.body {
        Some(body) => Request::with_body(
            inputs.url,
            inputs.method,
            inputs.headers,
            body.as_str(),
            inputs.timeout,
        )?,
        None => Request::new(inputs.url, inputs.method, inputs.headers, inputs.timeout)?,
    };

    let connector: Box<dyn Connector> = if inputs.proxy {
        Box::new(ProxyConnector::new(request.timeout))
    } else {
        Box::new(RegularConnector::new(request.timeout))
    };

    let requester = Requester::new(connector);

    let request_output = serde_json::to_value(&request)?;
    let response = requester.send_request(request)?;

    handle_output(response, request_output, inputs.verbose)
}
