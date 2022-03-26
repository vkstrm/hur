pub mod response;
pub mod request;
pub mod headers;
pub mod urldetails;

#[derive(serde::Serialize, Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    TRACE,
    PATCH,
    CONNECT,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(serde::Serialize, Debug)]
pub enum Scheme {
    HTTP,
    HTTPS
}