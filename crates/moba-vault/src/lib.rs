//! moba-vault crate.
//!
//! Master-password credential vault using Argon2id KDF + AES-256-GCM.
//! Secrets are zeroized on drop.
//!
//! See `docs/TASKS.md` for the current task ledger and `docs/PARITY.md`
//! for the feature-parity matrix.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod vault;

pub use vault::{Secret, Vault, VaultEntry, VaultError};
