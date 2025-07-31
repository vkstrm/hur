use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use native_tls::TlsConnector;
use std::time::Duration;

use crate::error;
use crate::error::Error;

pub trait Connector {
    fn http_request(&self, addr: SocketAddr, request_str: &str) -> Result<Vec<u8>, Error>;
    fn https_request(
        &self,
        addr: SocketAddr,
        domain: &str,
        request_str: &str,
    ) -> Result<Vec<u8>, Error>;
}

pub struct RegularConnector {
    timeout: u64,
}

impl RegularConnector {
    pub fn new(timeout: u64) -> Self {
        RegularConnector { timeout }
    }
}

impl Connector for RegularConnector {
    fn http_request(&self, addr: SocketAddr, request_str: &str) -> Result<Vec<u8>, Error> {
        http_request(addr, request_str, self.timeout)
    }

    fn https_request(
        &self,
        addr: SocketAddr,
        domain: &str,
        request_str: &str,
    ) -> Result<Vec<u8>, Error> {
        log::info!("Connecting to {}", addr.to_string());
        let stream = connect_timeout(&addr, self.timeout)?;
        tls_request(stream, domain, request_str.as_bytes())
    }
}

pub struct ProxyConnector {
    timeout: u64,
}

impl ProxyConnector {
    pub fn new(timeout: u64) -> Self {
        ProxyConnector { timeout }
    }
}

impl Connector for ProxyConnector {
    fn http_request(&self, addr: SocketAddr, request_str: &str) -> Result<Vec<u8>, Error> {
        http_request(addr, request_str, self.timeout)
    }

    fn https_request(
        &self,
        addr: SocketAddr,
        domain: &str,
        request_str: &str,
    ) -> Result<Vec<u8>, Error> {
        let mut stream = connect_timeout(&addr, self.timeout)?;
        connect_proxy(&mut stream, domain, addr)?;
        tls_request(stream, domain, request_str.as_bytes())
    }
}

fn connect_timeout(addr: &SocketAddr, timeout: u64) -> Result<TcpStream, Error> {
    let connection_duration = Duration::new(5, 0);
    let stream = TcpStream::connect_timeout(addr, connection_duration)?;

    stream.set_read_timeout(Some(Duration::new(timeout, 0)))?;
    stream.set_write_timeout(None)?;

    Ok(stream)
}

fn connect_proxy(
    stream: &mut TcpStream,
    domain: &str,
    proxy_addr: SocketAddr,
) -> Result<(), Error> {
    log::info!(
        "Performing CONNECT request to proxy {}",
        proxy_addr.to_string()
    );
    let mut connect_buffer: [u8; 39] = [0; 39];

    stream.write_all(connect_message(domain).as_bytes())?;
    stream.read_exact(&mut connect_buffer)?;

    if !connect_successful(&connect_buffer) {
        error!("connect request failed");
    }
    log::info!("CONNECT request to proxy was successful");
    Ok(())
}

fn connect_message(domain: &str) -> String {
    format!(
        "CONNECT {domain}:443 HTTP/1.1\r\nHost:{domain}\r\nConnection:keep-alive\r\n\r\n"
    )
}

fn connect_successful(buf: &[u8]) -> bool {
    buf.starts_with(b"HTTP/1.1 200") && buf.ends_with(b"\r\n\r\n")
}

fn tls_request(stream: TcpStream, domain: &str, request: &[u8]) -> Result<Vec<u8>, Error> {
    let tls_connector = TlsConnector::new()?;
    let mut stream = tls_connector.connect(domain, stream)?;
    let mut response_buffer: Vec<u8> = Vec::new();

    write_read(&mut stream, request, &mut response_buffer)?;
    Ok(response_buffer)
}

fn http_request(addr: SocketAddr, request_str: &str, timeout: u64) -> Result<Vec<u8>, Error> {
    log::info!("Connecting to {}", addr.to_string());
    let mut stream = connect_timeout(&addr, timeout)?;
    let mut response_buffer = Vec::new();

    write_read(&mut stream, request_str.as_bytes(), &mut response_buffer)?;
    Ok(response_buffer)
}

fn write_read<T>(stream: &mut T, message: &[u8], buffer: &mut Vec<u8>) -> Result<(), Error>
where
    T: Write + Read,
{
    stream.write_all(message)?;
    stream.read_to_end(buffer)?;
    Ok(())
}
