use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub discovery_port: u16,
    pub tcp_port_range: (u16, u16),
    pub username: String,
    pub network_timeout: u64, // seconds
    pub heartbeat_interval: u64, // seconds
}

impl Default for Config {
    fn default() -> Self {
        Self {
            discovery_port: 7878,
            tcp_port_range: (8000, 8100),
            username: whoami::username(),
            network_timeout: 10,
            heartbeat_interval: 30,
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_username(mut self, username: String) -> Self {
        self.username = username;
        self
    }
    
    pub fn find_available_discovery_port(&self) -> u16 {
        // Try the default port first, then try nearby ports
        for port in self.discovery_port..self.discovery_port + 10 {
            if let Ok(_) = std::net::UdpSocket::bind(("0.0.0.0", port)) {
                return port;
            }
        }
        // Fallback to any available port
        0
    }
}
