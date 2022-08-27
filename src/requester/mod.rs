use crate::error::Error;
use crate::error;
use crate::http::{request::Request, response::Response, Scheme};

use std::net::SocketAddr;

mod connector;
mod proxy;

type HttpsFunc = fn(server: SocketAddr, request: &str, domain: &str) -> Result<Vec<u8>, Error>;
type HttpFunc = fn(server: SocketAddr, request: &str) -> Result<Vec<u8>, Error>;

pub fn send_request(request: Request, allow_proxy: bool) -> Result<Response, Error> {
    match allow_proxy {
        true => try_for_proxy(request),
        false => match request.scheme {
            Scheme::HTTP => http(request),
            Scheme::HTTPS => https(request)
        }
    } 
}

fn try_for_proxy(request: Request) -> Result<Response, Error> {
    match proxy::should_proxy(&request)? {
        Some(servers) => match request.scheme {
            Scheme::HTTP => proxy_http(request, servers),
            Scheme::HTTPS => proxy_https(request, servers)
        },
        None => match request.scheme {
            Scheme::HTTP => http(request),
            Scheme::HTTPS => https(request)
        } 
    }
}

fn http(request: Request) -> Result<Response, Error> {
    let request_str = request.build();
    let servers = request.find_socket_addresses()?;
    internal_http(connector::http_request, servers, &request_str)
}

fn https(request: Request) -> Result<Response, Error> {
    let request_str = request.build();
    let servers = request.find_socket_addresses()?;
    internal_https(connector::https_request, servers, &request_str, &request.url.domain().unwrap())
}

fn proxy_http(request: Request, servers: Vec<SocketAddr>) -> Result<Response, Error> {
    let request_str = request.build_http_proxy();
    internal_http(connector::http_request, servers, &request_str)
}

fn proxy_https(request: Request, servers: Vec<SocketAddr>) -> Result<Response, Error> {
    let request_str = request.build();
    internal_https(connector::proxy_https_request, servers, &request_str, &request.url.domain().unwrap())
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


#[cfg(test)]
mod tests {
    use httptest::{Server, Expectation, matchers::*, responders::*};
    use url::Url;
    use super::*;
    use crate::http::{Method, headers::Headers};
    use serde_json;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct TestType {
        name: String,
        age: u32,
    }

    #[test]
    fn get_request_ok() {
        // Arrange
        let server = get_json_server();
        let uri = server.url("/foo");

        let url = Url::parse(&uri.to_string()).unwrap();
        let request = Request::new(url, Method::GET, Headers::new()).unwrap();

        // Act
        let response = send_request(request, false).unwrap();

        // Assert
        assert_eq!(response.status_code, 200);
        assert_eq!(response.headers.get("content-type").unwrap().first().unwrap(), "application/json");
        let body: TestType = serde_json::from_str(&response.body.unwrap()).unwrap();
        assert_eq!(body.name, "Bob");
        assert_eq!(body.age, 25);
    }

    fn get_json_server() -> Server {
        let server = Server::run();
        let responder = status_code(200)
            .body(r#"{"name":"Bob", "age":25}"#)
            .append_header("Content-Type", "application/json");

        let expectation = Expectation::matching(request::method_path("GET", "/foo"));
        server.expect(expectation.respond_with(responder));
        server
    }
}