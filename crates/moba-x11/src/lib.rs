//! moba-x11 crate.
//!
//! X11 forwarding configuration and display model.
//!
//! See `docs/TASKS.md` for the current task ledger and `docs/PARITY.md`
//! for the feature-parity matrix.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;

pub use config::{X11Display, X11ForwardConfig};
