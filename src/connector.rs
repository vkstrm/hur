use std::net::{SocketAddr,TcpStream};
use std::io::{Read, Write};

use native_tls::TlsConnector;

use super::error::Error;
use super::http;

pub struct Connector {
    tcp_stream: TcpStream,
}

impl Connector {
    pub fn new(addr: SocketAddr) -> Result<Connector, Error> {
        Ok(Connector {
            tcp_stream: TcpStream::connect(addr)?,
        })
    }

    pub fn send_http_request(self, request_str: &str) -> Result<http::response::Response, Error> {
        let mut response_buffer = vec![];
        match self.do_http_request(
                request_str.as_bytes(),
                &mut response_buffer) {
                    Ok(()) => http::response::Response::from_response(&response_buffer),
                    Err(why) => Err(why)
                }
    }

    pub fn send_https_request(self, request_str: &str, domain: &Option<String>) 
    -> Result<http::response::Response, Error> {
        let mut response_buffer = vec![];
        let domain = match domain {
            Some(domain) => domain,
            None => return Err(Error::new("Need domain for HTTPS"))
        };
        match self.do_https_request(
            domain,
            request_str.as_bytes(), 
            &mut response_buffer) {
                Ok(()) => match http::response::Response::from_response(&response_buffer) {
                        Ok(response) => Ok(response),
                        Err(why) => Err(why),
                    },
                Err(why) => Err(why)
            }
    }

    fn do_http_request(mut self, message: &[u8], buffer: &mut Vec<u8>) -> Result<(), Error> {
        write_read(&mut self.tcp_stream, message, buffer)
    }

    fn do_https_request(self, domain: &str, message: &[u8], buffer: &mut Vec<u8>) 
        -> Result<(), Error> {
        let connector = TlsConnector::new()?;
        let mut stream = connector.connect(domain, self.tcp_stream)?;
        write_read(&mut stream, message, buffer)
    }
}


fn write_read<T>(stream: &mut T, message: &[u8], buffer: &mut Vec<u8>) 
-> Result<(), Error> where T: Write + Read {
    stream.write_all(message)?;
    stream.read_to_end(buffer)?;
    Ok(())
}