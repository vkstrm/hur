use std::net::{SocketAddr, ToSocketAddrs};

mod error;
mod request;
mod connector;
mod input;

use error::Error;
use request::HttpRequest;

pub fn perform(args: &Vec<String>) {
    let input = match input::parse_args(&args) {
        Ok(input) => input,
        Err(why) => {
            eprintln!("{}", why);
            return
        }
    };

    let servers = match addr_from_url(&input.url) {
        Ok(servers) => servers,
        Err(why) => {
            eprintln!("{}", why);
            return
        }
    };
    println!("{:?}", servers);

    let request = HttpRequest::new(input.method, &input.url);
    let request_str = request.build();
    println!("{}", request_str);

    let mut response_buffer = vec![];
    match input.url.scheme() {
       "http" => connector::do_http_request(
           servers[0],
           request_str.as_bytes(),
           &mut response_buffer).unwrap(),
       "https" => connector::do_https_request(
           servers[0], 
           input.url.domain().unwrap(),
           &request_str.as_bytes(), 
           &mut response_buffer).unwrap(),
       _ => {},
    };

    println!("{}", String::from_utf8_lossy(&response_buffer));
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