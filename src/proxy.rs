use std::net::ToSocketAddrs;
use regex::Regex;

use crate::http;
use crate::error::Error;

pub fn should_proxy(request: &http::request::Request) -> Result<Option<Vec<std::net::SocketAddr>>, Error> {
    if let Some(no_proxy) = get_env("NO_PROXY") {
        let no_proxy_splits: Vec<&str> = no_proxy.split(",").collect();
        for no_proxy_entry in no_proxy_splits {
            if request.host == no_proxy_entry  {
                return Ok(None)
            }

            if is_ip_address(no_proxy_entry) && no_proxy_server(&request.servers, no_proxy_entry) {
                return Ok(None)
            }
        }
    }

    proxy(&request.scheme)
}

fn no_proxy_server(servers: &Vec<std::net::SocketAddr>, no_proxy: &str) -> bool {
    let no_proxy_splits: Vec<&str> = no_proxy.split(".").collect();
    for server in servers {
        let server_str = server.to_string();
        let server_splits: Vec<&str> = server_str.split(".").collect();

        let i = 0;
        while i < no_proxy_splits.len() &&  i < server_splits.len() {
            if no_proxy_splits[i] != "*" && no_proxy_splits[i] != server_splits[i] {
                return false
            }     
        }
    }

    return true
}

fn proxy(scheme: &http::Scheme) -> Result<Option<Vec<std::net::SocketAddr>>, Error> {
    let proxy_key: String;
    match scheme {
        http::Scheme::HTTP => {
            proxy_key = "HTTP_PROXY".to_string()
        },
        http::Scheme::HTTPS => {
            proxy_key = "HTTPS_PROXY".to_string()
        }
    }
    if let Some(proxy) = get_env(&proxy_key) {
        return Ok(Some(proxy_address(&proxy)?))
    }

    Ok(None)
}

fn proxy_address(proxy: &str) -> Result<Vec<std::net::SocketAddr>, Error> {
    let proxy = match url::Url::parse(proxy) {
        Ok(url) => url,
        Err(why) => return Err(Error::new(&why.to_string()))
    };
    let domain = match proxy.domain() {
        Some(domain) => domain,
        None => return Err(Error::new("no domain in proxy url"))
    };
    let port = match proxy.port() {
        Some(port) => port,
        None => return Err(Error::new("no port in proxy url"))
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
        Err(_) => {
            match std::env::var(env.to_uppercase()) {
                Ok(e) => Some(e),
                Err(_) => None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ip_address() {
        assert_eq!(is_ip_address("129.0.0.1"), true);
        assert_eq!(is_ip_address("129.0.*"), true);
        assert_eq!(is_ip_address("132.*"), true);
        assert_eq!(is_ip_address("localhost"), false);
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