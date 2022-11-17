use crate::error::Error;
use crate::error;
use crate::http::{request::Request, response::Response, Scheme};

pub mod connector;

use connector::Connector;

pub struct Requester {
    connector: Box<dyn Connector>
}

impl Requester {
    pub fn new(connector: Box<dyn Connector>) -> Self {
        Requester {
            connector
        }
    }

    pub fn send_request(&self, request: Request) -> Result<Response, Error> {
        self.match_schema(request)
    }
    
    // fn try_for_proxy(&self, request: Request) -> Result<Response, Error> {
    //     match proxy::should_proxy(&request)? {
    //         Some(servers) => match request.scheme {
    //             Scheme::HTTP => self.proxy_http(request, servers),
    //             Scheme::HTTPS => self.proxy_https(request, servers)
    //         },
    //         None => self.match_schema(request)
    //     }
    // }

    fn match_schema(&self, request: Request) -> Result<Response, Error> {
        match request.scheme {
            Scheme::HTTP => self.http(request),
            Scheme::HTTPS => self.https(request)
        } 
    }
    
    fn http(&self, request: Request) -> Result<Response, Error> {
        // let servers = request.find_socket_addresses()?;
        let request_str = request.build();
        for server in request.servers {
            let server_str = server.to_string();
            log::info!("Trying server {}", server_str);
            match self.connector.http_request(server, &request_str) {
                Ok(response) => return Response::from_buffer(&response),
                Err(err) => {
                    log::warn!("Request to {} failed with error {}", server_str, err);
                    continue;
                }
            }
        }
    
        error!("no server worked for request")
    }
    
    fn https(&self, request: Request) -> Result<Response, Error> {
        let request_str = request.build();
        // let servers = request.find_socket_addresses()?;
        for server in request.servers {
            let server_str = server.to_string();
            log::info!("Trying server {}", server_str);
            match self.connector.https_request(server, request.url.domain().unwrap(), &request_str) {
                Ok(response) => return Response::from_buffer(&response),
                Err(err) => {
                    log::warn!("Request to {} failed with error {}", server_str, err);
                    continue;
                }
            }
        }
    
        error!("no server worked for request")
    }
    
    // fn proxy_http(&self, request: Request, servers: Vec<SocketAddr>) -> Result<Response, Error> {
    //     let request_str = request.build_http_proxy();
    //     for server in servers {
    //         let server_str = server.to_string();
    //         log::info!("Trying server {}", server_str);
    //         match self.connector.http_request(server, &request_str) {
    //             Ok(response) => return Response::from_buffer(&response),
    //             Err(err) => {
    //                 log::warn!("Request to {} failed with error {}", server_str, err);
    //                 continue;
    //             }
    //         }
    //     }
    
    //     error!("no server worked for request")
    // }
    
    // fn proxy_https(&self, request: Request, servers: Vec<SocketAddr>) -> Result<Response, Error> {
    //     let request_str = request.build();
    //     for server in servers {
    //         let server_str = server.to_string();
    //         log::info!("Trying server {}", server_str);
    //         match self.connector.https_request(server, request.url.domain().unwrap(), &request_str) {
    //             Ok(response) => return Response::from_buffer(&response),
    //             Err(err) => {
    //                 log::warn!("Request to {} failed with error {}", server_str, err);
    //                 continue;
    //             }
    //         }
    //     }
    
    //     error!("no server worked for request")
    // }
}


// #[cfg(test)]
// mod tests {
//     use httptest::{Server, Expectation, matchers::*, responders::*};
//     use url::Url;
//     use super::*;
//     use crate::http::{Method, headers::Headers};
//     use serde_json;
//     use serde::{Deserialize, Serialize};

//     #[derive(Serialize, Deserialize)]
//     struct TestType {
//         name: String,
//         age: u32,
//     }

//     #[test]
//     fn get_request_ok() {
//         // Arrange
//         let server = get_json_server();
//         let uri = server.url("/foo");

//         let url = Url::parse(&uri.to_string()).unwrap();
//         let request = Request::new(url, Method::GET, Headers::new()).unwrap();

//         let requester = Requester::new(10);

//         // Act
//         let response = requester.send_request(request, false).unwrap();

//         // Assert
//         assert_eq!(response.status_code, 200);
//         assert_eq!(response.headers.get("content-type").unwrap().first().unwrap(), "application/json");
//         let body: TestType = serde_json::from_str(&response.body.unwrap()).unwrap();
//         assert_eq!(body.name, "Bob");
//         assert_eq!(body.age, 25);
//     }

//     fn get_json_server() -> Server {
//         let server = Server::run();
//         let responder = status_code(200)
//             .body(r#"{"name":"Bob", "age":25}"#)
//             .append_header("Content-Type", "application/json");

//         let expectation = Expectation::matching(request::method_path("GET", "/foo"));
//         server.expect(expectation.respond_with(responder));
//         server
//     }
// }