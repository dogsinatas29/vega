pub mod ssh;

use std::net::{SocketAddr, ToSocketAddrs};

#[derive(Debug, Clone)]
pub struct NetworkAddress {
    pub host: String,
    pub port: u16,
}

impl NetworkAddress {
    pub fn new(host: &str, port: u16) -> Self {
        // If host already contains a port, we strip it to avoid double-porting
        let clean_host = if host.contains(':') && !host.starts_with('[') {
             // Likely IPv4:Port
             host.split(':').next().unwrap_or(host).to_string()
        } else if host.starts_with('[') && host.contains("]:") {
             // Likely [IPv6]:Port
             host.split("]:").next().unwrap_or(host).trim_start_matches('[').to_string()
        } else {
             host.to_string()
        };

        Self {
            host: clean_host,
            port,
        }
    }

    pub fn to_socket_addr(&self) -> Result<SocketAddr, String> {
        let addr_str = format!("{}:{}", self.host, self.port);
        addr_str.to_socket_addrs()
            .map_err(|e| format!("Address Resolution Failed ({}): {}", addr_str, e))?
            .next()
            .ok_or_else(|| format!("No socket address found for {}", addr_str))
    }
}

impl std::fmt::Display for NetworkAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.host.contains(':') {
            // IPv6 display format
            write!(f, "[{}]:{}", self.host, self.port)
        } else {
            write!(f, "{}:{}", self.host, self.port)
        }
    }
}
#[derive(Debug, Clone)]
pub struct Target {
    pub addr: NetworkAddress,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl Target {
    pub fn new(host: &str, port: u16, user: Option<String>, password: Option<String>) -> Self {
        Self {
            addr: NetworkAddress::new(host, port),
            user,
            password,
        }
    }

    pub fn identifier(&self) -> String {
        self.addr.host.clone()
    }
}

impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(user) = &self.user {
            write!(f, "{}@{}", user, self.addr)
        } else {
            write!(f, "{}", self.addr)
        }
    }
}
