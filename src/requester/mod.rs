use std::io::{self, Write};

use crate::error;
use crate::error::Error;
use crate::http::{Scheme, request::Request, response::Response};
use crate::modes::RedirectMode;

pub mod connector;

use connector::Connector;
use url::Url;

pub struct Requester {
    connector: Box<dyn Connector>,
    redirect_mode: RedirectMode,
}

impl Requester {
    pub fn new(connector: Box<dyn Connector>, redirect_mode: RedirectMode) -> Self {
        Requester {
            connector,
            redirect_mode,
        }
    }
    pub fn do_request(&self, request: Request) -> Result<Response, Error> {
        let response = self.send_request(&request)?;
        match self.redirect_mode {
            RedirectMode::NoFollow => Ok(response),
            RedirectMode::Follow | RedirectMode::Interactive => match response.status_code {
                301 | 302 | 307 | 308 => {
                    let location = if let Some(location) = response.headers.get_first("location") {
                        location
                    } else {
                        error!("status code suggests redirect but Location header is not present")
                    };
                    let location_url = match Url::parse(&location) {
                        Ok(url) => url,
                        Err(why) => error!(&why.to_string()),
                    };

                    if let RedirectMode::Interactive = self.redirect_mode {
                        eprint!("Redirect to {location_url}? [Y, n]: ");
                        io::stdout().flush()?;
                        let mut answer = String::new();
                        match io::stdin().read_line(&mut answer) {
                            Ok(_) => {
                                let answer = answer.trim().to_lowercase();
                                if answer.is_empty() && !answer.starts_with("y") {
                                    return Ok(response);
                                }
                            }
                            Err(why) => {
                                error!(&why.to_string());
                            }
                        }
                    }

                    log::debug!("Following redirect to {}", location_url.as_str());
                    let request = Request::new(
                        location_url,
                        request.method,
                        request.headers,
                        Some(request.timeout),
                    )?;
                    self.send_request(&request)
                }
                _ => Ok(response),
            },
        }
    }

    fn send_request(&self, request: &Request) -> Result<Response, Error> {
        let request_str = request.build();
        for server in &request.servers {
            let server_str = server.to_string();
            log::debug!("Trying server {}", server_str);
            let result = match request.scheme {
                Scheme::Http => self.connector.http_request(server.to_owned(), &request_str),
                Scheme::Https => self.connector.https_request(
                    server.to_owned(),
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
        let request = Request::new(url, Method::Get, Headers::new(), None).unwrap();

        let requester = Requester::new(Box::new(RegularConnector::new(10)), RedirectMode::Follow);

        // Act
        let response = requester.do_request(request).unwrap();

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
