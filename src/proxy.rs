use regex::Regex;
use std::net::{SocketAddr, ToSocketAddrs};
use url::Url;

use crate::error;
use crate::error::Error;
use crate::http::{self, Scheme};

pub fn should_proxy(
    url: &Url,
    servers: &[SocketAddr],
    scheme: &Scheme,
) -> Result<Option<Vec<std::net::SocketAddr>>, Error> {
    if let Some(no_proxy) = get_env("NO_PROXY") {
        let no_proxy_splits: Vec<&str> = no_proxy.split(',').collect();
        for no_proxy_entry in no_proxy_splits {
            if url.host().unwrap().to_string() == no_proxy_entry {
                return Ok(None);
            }

            if is_ip_address(no_proxy_entry) && no_proxy_server(servers, no_proxy_entry) {
                return Ok(None);
            }
        }
    }

    proxy(scheme)
}

fn no_proxy_server(servers: &[std::net::SocketAddr], no_proxy: &str) -> bool {
    let no_proxy_splits: Vec<&str> = no_proxy.split('.').collect();
    for server in servers {
        let server_str = server.to_string();
        let server_splits: Vec<&str> = server_str.split('.').collect();

        let i = 0;
        while i < no_proxy_splits.len() && i < server_splits.len() {
            if no_proxy_splits[i] != "*" && no_proxy_splits[i] != server_splits[i] {
                return false;
            }
        }
    }

    true
}

fn proxy(scheme: &http::Scheme) -> Result<Option<Vec<std::net::SocketAddr>>, Error> {
    let proxy_key = match scheme {
        http::Scheme::Http => "HTTP_PROXY".to_string(),
        http::Scheme::Https => "HTTPS_PROXY".to_string(),
    };
    if let Some(proxy) = get_env(&proxy_key) {
        log::info!("Found proxy address {}", proxy);
        return Ok(Some(proxy_address(&proxy)?));
    }

    Ok(None)
}

fn proxy_address(proxy: &str) -> Result<Vec<std::net::SocketAddr>, Error> {
    let proxy = match url::Url::parse(proxy) {
        Ok(url) => url,
        Err(why) => error!(&why.to_string()),
    };
    let domain = match proxy.domain() {
        Some(domain) => domain,
        None => error!("no domain in proxy url"),
    };
    let port = match proxy.port() {
        Some(port) => port,
        None => error!("no port in proxy url"),
    };

    let proxy = format!("{}:{}", domain, port);
    Ok(proxy.to_socket_addrs()?.collect())
}

fn is_ip_address(ip: &str) -> bool {
    let re = Regex::new(r"(\d+\.?)+").unwrap();
    re.is_match(ip)
}

fn get_env(env: &str) -> Option<String> {
    match std::env::var(env.to_lowercase()) {
        Ok(e) => Some(e),
        Err(_) => match std::env::var(env.to_uppercase()) {
            Ok(e) => Some(e),
            Err(_) => None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ip_address() {
        assert!(is_ip_address("129.0.0.1"));
        assert!(is_ip_address("129.0.*"));
        assert!(is_ip_address("132.*"));
        assert!(!is_ip_address("localhost"));
    }

    #[test]
    fn test_get_env() {
        assert_eq!(get_env("TEST_ENV"), None);
        assert_eq!(get_env("test_env"), None);
        std::env::set_var("TEST_ENV", "ok");
        assert_eq!(get_env("TEST_ENV"), Some("ok".to_string()));
        assert_eq!(get_env("test_env"), Some("ok".to_string()));
    }
}
