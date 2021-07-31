use std::net::{SocketAddr,TcpStream};
use std::io::{Read, Write};

use native_tls::TlsConnector;

use super::error::Error;

pub fn do_http_request(addr: SocketAddr, message: &[u8], buffer: &mut Vec<u8>) -> Result<(), Error> {
    let mut stream = new_connection(addr)?;
    write_read(&mut stream, message, buffer)
}

pub fn do_https_request(addr: SocketAddr, domain: &str, message: &[u8], buffer: &mut Vec<u8>) 
-> Result<(), Error> {
    let mut stream = new_tls_connection(addr, domain)?;
    write_read(&mut stream, message, buffer)
}

fn new_connection(addr: std::net::SocketAddr) -> Result<TcpStream, Error>  {
    Ok(TcpStream::connect(addr)?)
}

fn new_tls_connection(addr: SocketAddr, domain: &str) -> Result<native_tls::TlsStream<TcpStream>, Error>  {
    let stream = TcpStream::connect(addr)?;
    let connector = TlsConnector::new()?;
    Ok(connector.connect(domain, stream)?)
}

fn write_read<T>(stream: &mut T, message: &[u8], buffer: &mut Vec<u8>) 
-> Result<(), Error> where T: Write + Read {
    stream.write_all(message)?;
    stream.read_to_end(buffer)?;
    Ok(())
}