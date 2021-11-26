mod error;
mod http;
mod connector;
mod inout;

use std::net::ToSocketAddrs;

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

    let response = match proxy()? {
        Some(addrs) => {
            match request.scheme.as_str() {
                "http" => connector::http_request(addrs[0], &request.build_http_proxy())?,
                "https" => {
                    let domain = request.domain.as_ref().unwrap().clone();
                    connector::proxy_https_request(addrs[0], &domain, &request.build())?
                },
                _ => Err(Error::new("only http/s supported"))?
            }
        },
        None => {
            let domain = request.domain.as_ref().ok_or("No domain").unwrap().clone();
            match request.scheme.as_str() {
                "http" => connector::http_request(request.servers[0], &request.build())?,
                "https" => connector::https_request(request.servers[0], &domain,&request.build())?, 
                _ => Err(Error::new("only http/s supported"))?
            }
        }
    };

    handle_output(&response, &request, inout.output);

    Ok(())
}

fn proxy() -> Result<Option<Vec<std::net::SocketAddr>>, Error> {
    // TODO Check https proxy, check no_proxy
    match std::env::var("HTTP_PROXY") {
        Ok(proxy) => {
            Ok(Some(proxy_address(&proxy)?))
        },
        Err(_) => Ok(None),
    }
}

fn proxy_address(proxy: &str) -> Result<Vec<std::net::SocketAddr>, Error> {
    let proxy = match url::Url::parse(proxy) {
        Ok(url) => url,
        Err(why) => return Err(Error::new(&why.to_string()))
    };
    let domain = match proxy.domain() {
        Some(domain) => domain,
        None => return Err(Error::new("no domain in proxy url"))
    };
    let port = match proxy.port() {
        Some(port) => port,
        None => return Err(Error::new("no port in proxy url"))
    };

    let proxy = format!("{}:{}", domain, port);
    Ok(proxy.to_socket_addrs()?.collect())
}

fn handle_output(response: &http::response::Response, request: &http::request::Request, output: inout::Output) {
    if output.verbose {
        print_verbose(response, request);
    } else if let Some(h) = output.query_header {
        query_header(&h, &response.headers)
    } else {
        match &response.body {
            Some(body) => println!("{}", body),
            None => {},
        }
    }
}

// TODO Return result
fn print_verbose(response: &http::response::Response, request: &http::request::Request) {
    let output_json = OutputJson {
        request,
        response
    };
    let json = serde_json::to_string_pretty(&output_json).unwrap(); // Dont unwrap!
    println!("{}", json);
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