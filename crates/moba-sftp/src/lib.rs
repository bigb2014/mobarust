//! moba-sftp crate.
//!
//! SFTP client and remote filesystem model.
//!
//! See `docs/TASKS.md` for the current task ledger and `docs/PARITY.md`
//! for the feature-parity matrix.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod fs;

pub use fs::{DirListing, RemoteEntry};
