use std::net::{SocketAddr,TcpStream};
use std::io::{Read, Write};

use native_tls::TlsConnector;

use super::error::Error;
use super::http;

type Response = http::response::Response;


pub fn http_request(addr: SocketAddr, request_str: &str) -> Result<Response, Error> {
    log::info!("Connecting to {}", addr.to_string());
    let mut stream = TcpStream::connect(addr)?;
    let mut response_buffer = Vec::new();
 
    do_http_request(&mut stream,request_str.as_bytes(), &mut response_buffer)?;
    Response::from_response(&response_buffer)
}

pub fn https_request(addr: SocketAddr, domain: &str, request_str: &str) -> Result<Response, Error> {
    log::info!("Connecting to {}", addr.to_string());
    let stream = TcpStream::connect(addr)?;
    let mut response_buffer = Vec::new();

    do_https_request(stream, domain, request_str.as_bytes(), &mut response_buffer)?;
    Response::from_response(&response_buffer)
}

pub fn proxy_https_request(proxy_addr: SocketAddr, domain: &str, request_str: &str) -> Result<Response, Error> {
    log::info!("Performing CONNECT request to proxy {}", proxy_addr.to_string());
    let mut connect_buffer: [u8; 39] = [0; 39];
    let mut stream = TcpStream::connect(proxy_addr)?;
    let connect_message = format!("CONNECT {0}:443 HTTP/1.1\r\nHost:{0}\r\nConnection:keep-alive\r\n\r\n", domain); 

    stream.write(connect_message.as_bytes())?;
    stream.read(&mut connect_buffer)?;
    if !connect_buffer.starts_with(b"HTTP/1.1 200") && !connect_buffer.ends_with(b"\r\n\r\n") {
        return Err(Error::new("connect request failed"));
    }
    log::info!("CONNECT request to proxy was succesful");

    tls_request(stream, domain, request_str.as_bytes())
}

fn tls_request(stream: TcpStream, domain: &str, request: &[u8]) -> Result<Response, Error> {
    let tls_connector = TlsConnector::new()?;
    let mut stream = tls_connector.connect(domain, stream)?;
    let mut request_buffer: Vec<u8> = Vec::new();

    write_read(&mut stream, &request, &mut request_buffer)?;
    Response::from_response(&request_buffer)
}

fn do_http_request(stream: &mut TcpStream, message: &[u8], buffer: &mut Vec<u8>) -> Result<(), Error> {
    write_read(stream, message, buffer)
}

fn do_https_request(stream: TcpStream, domain: &str, message: &[u8], buffer: &mut Vec<u8>) -> Result<(), Error> {
    let tls_connector = TlsConnector::new()?;
    let mut stream = tls_connector.connect(domain, stream)?;
    write_read(&mut stream, message, buffer)
}

fn write_read<T>(stream: &mut T, message: &[u8], buffer: &mut Vec<u8>) -> Result<(), Error> where T: Write + Read {
    stream.write_all(message)?;
    stream.read_to_end(buffer)?;
    Ok(())
}