#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl std::error::Error for Error {}

impl Error {
    pub fn new(message: &str) -> Error {
        Error{
            message: String::from(message),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "error performing request: {}", self.message)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error {
            message: err.to_string(),
        }
    }
}

impl From<native_tls::Error> for Error {
    fn from(err: native_tls::Error) -> Self {
        Error {
            message: err.to_string(),
        }
    }
}

impl From<native_tls::HandshakeError<std::net::TcpStream>> for Error {
    fn from(err: native_tls::HandshakeError<std::net::TcpStream>) -> Self {
        Error {
            message: err.to_string(),
        }
    }
}
