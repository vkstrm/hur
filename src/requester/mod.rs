use crate::error::Error;
use crate::error;
use crate::http::{request::Request, response::Response};

use std::net::SocketAddr;

mod connector;

type HttpsFunc = fn(server: SocketAddr, request: &str, domain: &str) -> Result<Vec<u8>, Error>;
type HttpFunc = fn(server: SocketAddr, request: &str) -> Result<Vec<u8>, Error>;

pub fn http(request: Request) -> Result<Response, Error> {
    let request_str = request.build();
    internal_http(connector::http_request, request.servers, &request_str)
}

pub fn https(request: Request) -> Result<Response, Error> {
    let request_str = request.build();
    internal_https(connector::https_request, request.servers, &request_str, &request.domain.unwrap())
}

pub fn proxy_http(request: Request, servers: Vec<SocketAddr>) -> Result<Response, Error> {
    let request_str = request.build_http_proxy();
    internal_http(connector::http_request, servers, &request_str)
}

pub fn proxy_https(request: Request, servers: Vec<SocketAddr>) -> Result<Response, Error> {
    let request_str = request.build();
    internal_https(connector::proxy_https_request, servers, &request_str, &request.domain.unwrap())
}

fn internal_https(func: HttpsFunc, servers: Vec<SocketAddr>, request: &str, domain: &str) -> Result<Response, Error> {
    for server in servers {
        let server_str = server.to_string();
        log::info!("Trying server {}", server_str);
        match func(server, domain, request) {
            Ok(response) => return Response::from_buffer(&response),
            Err(err) => {
                log::warn!("Request to {} failed with error {}", server_str, err);
                continue;
            }
        }
    }

    error!("no server worked for request")
}

fn internal_http(func: HttpFunc, servers: Vec<SocketAddr>, request: &str) -> Result<Response, Error> {
    for server in servers {
        let server_str = server.to_string();
        log::info!("Trying server {}", server_str);
        match func(server, request) {
            Ok(response) => return Response::from_buffer(&response),
            Err(err) => {
                log::warn!("Request to {} failed with error {}", server_str, err);
                continue;
            }
        }
    }

    error!("no server worked for request")
}