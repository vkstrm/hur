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

    let response = match proxy(&request.scheme)? {
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

    handle_output(&response, &request, inout.output)
}

fn proxy(schema: &http::Scheme) -> Result<Option<Vec<std::net::SocketAddr>>, Error> {
    let proxy_key: String;
    match schema {
        http::Scheme::HTTP => {
            proxy_key = "HTTP_PROXY".to_string()
        },
        http::Scheme::HTTPS => {
            proxy_key = "HTTPS_PROXY".to_string()
        }
    }
    match std::env::var(proxy_key) {
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

fn handle_output(response: &http::response::Response, request: &http::request::Request, output: inout::Output) -> Result<(), Error> {
    if output.verbose {
        print_verbose(response, request)?;
    } else if let Some(h) = output.query_header {
        query_header(&h, &response.headers)
    } else {
        match &response.body {
            Some(body) => println!("{}", body),
            None => {},
        }
    }
    Ok(())
}

fn print_verbose(response: &http::response::Response, request: &http::request::Request) -> Result<(), Error>{
    let output_json = OutputJson {
        request,
        response
    };
    let json = serde_json::to_string_pretty(&output_json)?;
    println!("{}", json);
    Ok(())
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