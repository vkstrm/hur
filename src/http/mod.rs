pub mod response;
pub mod request;
pub mod headers;

#[derive(Debug)]
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

pub trait Printer {
    fn print_headers(&self, verbose: bool);
    fn print_body(&self, verbose: bool);
}