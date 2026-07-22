//! moba-telnet crate.
//!
//! Telnet/Rlogin/Rsh protocol implementation.
//!
//! See `docs/TASKS.md` for the current task ledger and `docs/PARITY.md`
//! for the feature-parity matrix.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod protocol;

pub use protocol::{TelnetCommand, TelnetParser};
