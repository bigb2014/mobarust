//! TCP-based "ping" utility.
//!
//! Raw ICMP sockets require elevated privileges, which is not practical
//! for a user-space terminal tool.  Instead [`Pinger`] performs a TCP
//! connect to port 80 (or a caller-supplied port) and measures the
//! round-trip time of the connect attempt.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use tokio::net::TcpStream;

use crate::NetError;

/// Default TCP port used by [`Pinger::ping`] when no port is supplied.
pub const DEFAULT_PING_PORT: u16 = 80;

/// Result of a successful TCP ping.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PingResult {
    /// The host that was probed.
    pub host: String,
    /// Measured round-trip time in milliseconds.
    pub rtt_ms: f64,
    /// TCP TTL reported by the OS for the connection (may be `None`
    /// when the information is unavailable on the current platform).
    pub ttl: Option<u32>,
}

/// A TCP-connect based ping utility.
///
/// ```no_run
/// # async fn run() -> Result<(), moba_net::NetError> {
/// use moba_net::Pinger;
/// use std::time::Duration;
///
/// let pinger = Pinger::new();
/// let result = pinger.ping("example.com", Duration::from_secs(3)).await?;
/// println!("{} ms", result.rtt_ms);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct Pinger;

impl Pinger {
    /// Create a new [`Pinger`].
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Perform a TCP-connect ping to `host` on the default port
    /// ([`DEFAULT_PING_PORT`]).
    ///
    /// The round-trip time is the wall-clock duration of the connect
    /// attempt.  A [`NetError::Timeout`] is returned when the connect
    /// does not complete within `timeout`.
    pub async fn ping(&self, host: &str, timeout: Duration) -> Result<PingResult, NetError> {
        self.ping_port(host, DEFAULT_PING_PORT, timeout).await
    }

    /// Perform a TCP-connect ping to `host` on a specific `port`.
    ///
    /// This is useful when port 80 is filtered but another service port
    /// (e.g. 22 or 443) is reachable.
    pub async fn ping_port(
        &self,
        host: &str,
        port: u16,
        timeout: Duration,
    ) -> Result<PingResult, NetError> {
        let addr = format!("{host}:{port}");
        let start = Instant::now();

        let connect_fut = async {
            let sock = addr.parse::<SocketAddr>();
            match sock {
                Ok(sa) => TcpStream::connect(sa).await,
                Err(_) => TcpStream::connect(&addr).await,
            }
        };

        let stream = match tokio::time::timeout(timeout, connect_fut).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                // Classify common errors.
                if e.kind() == std::io::ErrorKind::ConnectionRefused {
                    return Err(NetError::ConnectionRefused);
                }
                if e.kind() == std::io::ErrorKind::TimedOut {
                    return Err(NetError::Timeout);
                }
                return Err(NetError::IoError(e));
            }
            Err(_) => return Err(NetError::Timeout),
        };

        let elapsed = start.elapsed();
        let rtt_ms = elapsed.as_secs_f64() * 1000.0;

        // Retrieve TTL when available.  On platforms where the socket
        // option is not supported, we store `None`.
        let ttl = stream_ttl(&stream);

        Ok(PingResult {
            host: host.to_string(),
            rtt_ms,
            ttl,
        })
    }
}

/// Attempt to read the TTL from a connected `TcpStream`.
///
/// This uses the platform-specific socket option and may return `None`
/// when the option is unavailable.
fn stream_ttl(stream: &TcpStream) -> Option<u32> {
    // `socket2` would be the idiomatic way, but to avoid pulling in
    // another dependency we rely on the standard library's
    // `TcpStream::ttl()` via the tokio wrapper.  Tokio exposes
    // `ttl()` only behind a feature; if it is not available we fall
    // back to `None`.
    stream_ttl_inner(stream).ok()
}

/// Inner helper for [`stream_ttl`]; isolated so the `Option` mapping
/// stays clean.
fn stream_ttl_inner(stream: &TcpStream) -> Result<u32, std::io::Error> {
    // Tokio's `TcpStream` exposes `ttl()` directly on all platforms.
    stream.ttl()
}
