use std::net::{SocketAddr, ToSocketAddrs};

mod error;
mod http;
mod connector;
mod input;

use error::Error;
use http::request::Request;
use http::Printer;

pub fn perform(args: &Vec<String>) {
    let input = input::parse_args(&args);
    let mut request = match Request::new(input.method, &input.url) {
        Ok(request) => request,
        Err(why) => {
            eprintln!("{}", why);
            return
        }
    };

    if let Some(headers) = input.headers {
        request.headers.append(headers);
    }
    if let Some(body) = input.body {
        request.set_body(body);
        if input.json {
            request.headers.add("Content-Type", "application/json")
        }
    }
    let request_str = request.build();
    if input.verbose {
        request.print_headers(input.verbose);
        request.print_body(input.verbose);
    } else if input.raw {
        println!("--- HTTP Request ---");
        println!("{}", request_str);
    }

    let servers = match addr_from_url(&request.url) {
        Ok(servers) => servers,
        Err(why) => {
            eprintln!("{}", why);
            return
        }
    };

    let mut response_buffer = vec![];
    match request.url.scheme() {
       "http" => connector::do_http_request(
           servers[0],
           request_str.as_bytes(),
           &mut response_buffer).unwrap(),
       "https" => connector::do_https_request(
           servers[0], 
           request.url.domain().unwrap(),
           &request_str.as_bytes(), 
           &mut response_buffer).unwrap(),
       _ => {},
    };

    match http::response::Response::from_response(&response_buffer) {
        Ok(response) => {
            if input.raw {
                match response.raw {
                    Some(body) => {
                        println!("--- Response HTTP ---");
                        println!("{}", body);
                        println!("---");
                    }
                    None => {},
                }
            } else {
                response.print_headers(input.verbose);
                response.print_body(input.verbose);
            }
        },
        Err(why) => eprintln!("{}", why),
    }
}

fn addr_from_url(url: &url::Url) -> Result<Vec<SocketAddr>, Error> {
    let mut server_details = String::new();
    match url.domain() {
        Some(_) => server_details.push_str(url.domain().unwrap()),
        None => server_details.push_str(url.host_str().unwrap()), 
    };
    server_details.push(':');
    match url.port() {
        Some(port) => server_details.push_str(&port.to_string()),
        None => {
            match url.scheme() {
                "https" => server_details.push_str("443"),
                "http" => server_details.push_str("80"),
                _ => return Err(Error::new("only support http/s"))
            }
        }
    }
    Ok(server_details.to_socket_addrs()?.collect())
}