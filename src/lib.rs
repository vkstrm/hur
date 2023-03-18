mod cli;
mod error;
mod http;
mod logs;
mod proxy;
mod requester;

use cli::output::handle_output;
use error::Error;
use requester::connector::{Connector, ProxyConnector, RegularConnector};
use requester::Requester;

pub fn handle(args: Vec<String>) {
    match handle_args(args) {
        Ok(()) => {}
        Err(error) => eprintln!("{}", error),
    }
}

fn handle_args(args: Vec<String>) -> Result<(), Error> {
    let (request, verbose) = cli::create_request(args)?;

    let connector: Box<dyn Connector> = if request.proxy {
        Box::new(ProxyConnector::new(request.timeout))
    } else {
        Box::new(RegularConnector::new(request.timeout))
    };

    let requester = Requester::new(connector);

    let request_output = serde_json::to_value(&request)?;
    let response = requester.send_request(request)?;

    handle_output(response, request_output, verbose)
}
