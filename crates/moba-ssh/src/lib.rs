//! moba-ssh crate.
//!
//! SSH client for MobaRust built on [`russh`]: password, public-key, and agent
//! authentication; known-hosts verification (TOFU); keepalive; and
//! terminal-over-SSH (PTY) support. Also includes the tunnel manager
//! for local/remote/dynamic (SOCKS) port forwarding.
//!
//! See `docs/TASKS.md` for the current task ledger and `docs/PARITY.md`
//! for the feature-parity matrix.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod client;
mod error;
mod known_hosts;
pub mod tunnels;

pub use client::{ConnectedClient, SshChannel, SshClient};
pub use error::SshError;
pub use known_hosts::{KnownHostResult, KnownHosts};
pub use tunnels::{ForwardType, TunnelManager, TunnelRule};
