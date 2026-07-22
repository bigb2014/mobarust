//! Network utilities crate for MobaRust.
//!
//! Provides lightweight network diagnostic helpers — port scanning,
//! TCP-based "ping", and shared error types.  See `docs/TASKS.md` for
//! the current task ledger and `docs/PARITY.md` for the feature-parity
//! matrix.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ping;
pub mod port_scan;

// Re-export the primary public items at the crate root for convenience.
pub use ping::{PingResult, Pinger};
pub use port_scan::PortScanner;

/// Errors that can arise from network operations.
#[derive(Debug, thiserror::Error)]
pub enum NetError {
    /// Wraps a [`std::io::Error`] from the underlying system call.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// The operation exceeded its allotted timeout.
    #[error("operation timed out")]
    Timeout,

    /// The remote endpoint actively refused the connection.
    #[error("connection refused")]
    ConnectionRefused,

    /// A DNS resolution failure with a descriptive message.
    #[error("DNS error: {0}")]
    DnsError(String),
}
