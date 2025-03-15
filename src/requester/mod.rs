use crate::error;
use crate::error::Error;
use crate::http::{Scheme, request::Request, response::Response};

pub mod connector;

use connector::Connector;

pub struct Requester {
    connector: Box<dyn Connector>,
}

impl Requester {
    pub fn new(connector: Box<dyn Connector>) -> Self {
        Requester { connector }
    }

    pub fn send_request(&self, request: Request) -> Result<Response, Error> {
        let request_str = request.build();
        for server in request.servers {
            let server_str = server.to_string();
            log::info!("Trying server {}", server_str);
            let result = match request.scheme {
                Scheme::Http => self.connector.http_request(server, &request_str),
                Scheme::Https => self.connector.https_request(
                    server,
                    request.url.domain().unwrap(),
                    &request_str,
                ),
            };
            match result {
                Ok(response) => return Response::from_buffer(&response),
                Err(err) => {
                    log::warn!("Request to {} failed with error {}", server_str, err);
                    continue;
                }
            }
        }

        error!("no server worked for request")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        http::{Method, headers::Headers},
        requester::connector::RegularConnector,
    };
    use httptest::{Expectation, Server, matchers::*, responders::*};
    use serde::{Deserialize, Serialize};

    use url::Url;

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
        let request = Request::new(url, Method::Get, Headers::new()).unwrap();

        let requester = Requester::new(Box::new(RegularConnector::new(10)));

        // Act
        let response = requester.send_request(request).unwrap();

        // Assert
        assert_eq!(response.status_code, 200);
        assert_eq!(
            response
                .headers
                .get("content-type")
                .unwrap()
                .first()
                .unwrap(),
            "application/json"
        );
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
