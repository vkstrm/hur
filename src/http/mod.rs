pub mod response;
pub mod request;
pub mod headers;

#[derive(serde::Serialize, Debug)]
pub enum Method {
    GET,
    POST,
    PUT,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}