//! Unit tests for moba-net.
//!
//! These tests exercise deterministic logic only (port lists, error
//! messages).  No actual network calls are performed.

use moba_net::{NetError, PortScanner};

/// The well-known common-ports list returned by [`PortScanner::common_ports`].
#[test]
fn common_ports_list_is_correct() {
    let ports = PortScanner::common_ports();
    assert_eq!(ports, vec![22, 80, 443, 3389, 5900, 8080]);
}

/// `generate_range` must produce every port in `[start, end]` inclusive.
#[test]
fn port_range_generates_all_ports() {
    let range = PortScanner::generate_range(80, 85);
    assert_eq!(range, vec![80, 81, 82, 83, 84, 85]);
}

/// `generate_range` with start == end yields a single element.
#[test]
fn port_range_single_port() {
    let range = PortScanner::generate_range(443, 443);
    assert_eq!(range, vec![443]);
}

/// `NetError::Timeout` display message.
#[test]
fn net_error_display_timeout() {
    let err = NetError::Timeout;
    assert_eq!(err.to_string(), "operation timed out");
}

/// `NetError::ConnectionRefused` display message.
#[test]
fn net_error_display_refused() {
    let err = NetError::ConnectionRefused;
    assert_eq!(err.to_string(), "connection refused");
}

/// `NetError::DnsError` display message includes the inner detail.
#[test]
fn net_error_display_dns() {
    let err = NetError::DnsError("nx-domain".to_string());
    assert_eq!(err.to_string(), "DNS error: nx-domain");
}

/// `NetError::IoError` forwards the underlying `std::io::Error` message.
#[test]
fn net_error_display_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    let err = NetError::IoError(io_err);
    assert!(err.to_string().contains("missing"));
}
