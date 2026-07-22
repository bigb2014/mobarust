//! moba-serial crate.
//!
//! Provides the serial port configuration model (baud rate, data bits,
//! parity, stop bits, flow control) used for serial terminal sessions.
//!
//! See `docs/TASKS.md` for the current task ledger and `docs/PARITY.md`
//! for the feature-parity matrix.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;
