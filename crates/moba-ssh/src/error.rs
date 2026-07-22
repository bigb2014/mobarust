//! Error types for the `moba-ssh` crate.

use std::io;

use thiserror::Error;

/// Errors that can arise during SSH connection, authentication, channel
/// operation, or host-key verification.
#[derive(Debug, Error)]
pub enum SshError {
    /// Failed to establish a TCP or SSH transport connection to the host.
    #[error("SSH connection error: {0}")]
    ConnectError(String),

    /// Authentication failed (bad password, rejected key, agent error).
    #[error("SSH authentication error: {0}")]
    AuthError(String),

    /// A channel-level operation (PTY, exec, write, resize) failed.
    #[error("SSH channel error: {0}")]
    ChannelError(String),

    /// An underlying I/O error (file read/write for keys, known_hosts, etc.).
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// Server host key did not match the known_hosts entry or was rejected.
    #[error("SSH host key error: {0}")]
    HostKeyError(String),

    /// Failed to load or parse a private/public key file.
    #[error("SSH key load error: {0}")]
    KeyLoadError(String),
}
