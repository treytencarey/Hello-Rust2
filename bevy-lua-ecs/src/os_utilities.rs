// Generic OS utilities for Lua scripts
// These provide low-level OS operations that Lua can compose into higher-level functionality

use std::net::{SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};

/// Bind a UDP socket to the given address
/// Returns the socket on success, or an error message
pub fn bind_udp_socket(addr: &str) -> Result<UdpSocket, String> {
    UdpSocket::bind(addr).map_err(|e| format!("Failed to bind socket to {}: {}", addr, e))
}

/// Get current system time as milliseconds since UNIX epoch
pub fn current_time_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_millis()
}

/// Parse a socket address string (e.g., "127.0.0.1:5000")
pub fn parse_socket_addr(addr: &str) -> Result<SocketAddr, String> {
    addr.parse()
        .map_err(|e| format!("Invalid socket address '{}': {}", addr, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_socket_addr() {
        assert!(parse_socket_addr("127.0.0.1:5000").is_ok());
        assert!(parse_socket_addr("0.0.0.0:8080").is_ok());
        assert!(parse_socket_addr("invalid").is_err());
    }

    #[test]
    fn test_current_time() {
        let time = current_time_millis();
        assert!(time > 0);
    }
}
