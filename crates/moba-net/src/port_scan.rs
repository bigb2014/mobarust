//! Port-scanning utilities.
//!
//! [`PortScanner`] offers simple async TCP connect probes.  Because raw
//! ICMP and SYN scanning require elevated privileges, all checks are
//! performed with a regular `TcpStream::connect`, which is sufficient
//! for the "is this port open?" question that MobaRust needs.

use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::TcpStream;

/// The list of ports that [`PortScanner::scan_common`] probes by default.
///
/// These cover the most frequently used remote-access and web services:
/// SSH (22), HTTP (80), HTTPS (443), RDP (3389), VNC (5900) and a
/// common alternate HTTP port (8080).
pub const COMMON_PORTS: &[u16] = &[22, 80, 443, 3389, 5900, 8080];

/// A simple TCP-connect based port scanner.
///
/// All methods are async and accept a per-probe [`Duration`] timeout so
/// that unreachable hosts do not stall the caller.
#[derive(Debug, Clone, Default)]
pub struct PortScanner;

impl PortScanner {
    /// Create a new [`PortScanner`].
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Return the canonical list of common ports scanned by
    /// [`Self::scan_common`].
    ///
    /// This is a static list and is safe to call from any context -- it
    /// performs no I/O.
    #[must_use]
    pub fn common_ports() -> Vec<u16> {
        COMMON_PORTS.to_vec()
    }

    /// Generate the inclusive range `[start, end]` of ports.
    ///
    /// If `start > end` the returned slice is empty, matching the
    /// semantics of a half-open interval.  No allocation occurs when
    /// the range is empty.
    #[must_use]
    pub fn generate_range(start: u16, end: u16) -> Vec<u16> {
        if start > end {
            return Vec::new();
        }
        (start..=end).collect()
    }

    /// Probe a single `(host, port)` pair using a TCP connect.
    ///
    /// Returns `true` when the connection completes within `timeout`,
    /// `false` on any error or timeout.  The function never panics --
    /// callers should treat a `false` result as "not reachable".
    ///
    /// # Arguments
    /// * `host` -- hostname or IP literal (e.g. `"192.168.1.1"`).
    /// * `port` -- target TCP port.
    /// * `timeout` -- maximum time to wait for the connect to finish.
    pub async fn scan_port(&self, host: &str, port: u16, timeout: Duration) -> bool {
        let addr = format!("{host}:{port}");
        let probe = async {
            // Try literal parse first (fast path), fall back to DNS.
            let sock = addr.parse::<SocketAddr>();
            match sock {
                Ok(sa) => TcpStream::connect(sa).await,
                Err(_) => TcpStream::connect(&addr).await,
            }
        };
        match tokio::time::timeout(timeout, probe).await {
            Ok(Ok(_stream)) => true,
            Ok(Err(_)) => false,
            Err(_) => false,
        }
    }

    /// Scan an inclusive port range `[start, end]` on `host`.
    ///
    /// Returns the sorted list of ports that responded within the
    /// per-probe `timeout`.  An empty result means no ports were open.
    pub async fn scan_range(
        &self,
        host: &str,
        start: u16,
        end: u16,
        timeout: Duration,
    ) -> Vec<u16> {
        let ports = Self::generate_range(start, end);
        // Run probes concurrently for responsiveness.
        let mut handles = Vec::with_capacity(ports.len());
        for port in ports {
            let host_owned = host.to_string();
            handles.push(tokio::spawn(async move {
                let scanner = PortScanner::new();
                let open = scanner.scan_port(&host_owned, port, timeout).await;
                if open {
                    Some(port)
                } else {
                    None
                }
            }));
        }
        let mut open_ports = Vec::new();
        for handle in handles {
            if let Ok(Some(port)) = handle.await {
                open_ports.push(port);
            }
        }
        open_ports.sort_unstable();
        open_ports
    }

    /// Scan the [`COMMON_PORTS`] list on `host`.
    ///
    /// Convenience wrapper around [`Self::scan_range`] using the static
    /// common-ports list.
    pub async fn scan_common(&self, host: &str, timeout: Duration) -> Vec<u16> {
        let ports = Self::common_ports();
        // Run probes concurrently for responsiveness.
        let mut handles = Vec::with_capacity(ports.len());
        for port in ports {
            let host_owned = host.to_string();
            handles.push(tokio::spawn(async move {
                let scanner = PortScanner::new();
                let open = scanner.scan_port(&host_owned, port, timeout).await;
                if open {
                    Some(port)
                } else {
                    None
                }
            }));
        }
        let mut open_ports = Vec::new();
        for handle in handles {
            if let Ok(Some(port)) = handle.await {
                open_ports.push(port);
            }
        }
        open_ports.sort_unstable();
        open_ports
    }
}
