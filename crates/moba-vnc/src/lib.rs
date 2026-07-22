//! moba-vnc crate.
//!
//! VNC client configuration model.
//!
//! See `docs/TASKS.md` for the current task ledger and `docs/PARITY.md`
//! for the feature-parity matrix.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod config;

pub use config::{PixelFormat, VncConfig};
