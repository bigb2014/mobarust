//! Known-hosts store for TOFU (trust-on-first-use) host-key verification.
//!
//! The store maps a host identifier (e.g. `host:port` or bare `host`) to the
//! fingerprint of the server's public key. On the first connection the
//! fingerprint is recorded; on subsequent connections the stored fingerprint
//! is compared to the one presented by the server. A mismatch signals a
//! possible man-in-the-middle attack.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::SshError;

/// Result of verifying a server's host key against the known-hosts store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KnownHostResult {
    /// The presented fingerprint matches the stored one.
    Trusted,
    /// A fingerprint is stored for this host but the presented one differs.
    Mismatch,
    /// No fingerprint is stored for this host (first contact / TOFU case).
    Unknown,
}

/// A map of host identifiers to public-key fingerprints.
///
/// Serialized as a JSON object (`host -> fingerprint`) so it can be persisted
/// to disk and inspected by humans.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct KnownHosts {
    hosts: HashMap<String, String>,
}

impl KnownHosts {
    /// Create an empty known-hosts store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record or overwrite the fingerprint for `host`.
    ///
    /// Call this on first contact (TOFU) or when the operator has confirmed a
    /// legitimately changed key.
    pub fn add(&mut self, host: &str, key_fingerprint: &str) {
        self.hosts
            .insert(host.to_string(), key_fingerprint.to_string());
    }

    /// Verify `key_fingerprint` against the stored entry for `host`.
    ///
    /// Returns [`KnownHostResult::Unknown`] when the host has no stored entry,
    /// [`KnownHostResult::Trusted`] when the fingerprints match, and
    /// [`KnownHostResult::Mismatch`] when they differ.
    #[must_use]
    pub fn verify(&self, host: &str, key_fingerprint: &str) -> KnownHostResult {
        match self.hosts.get(host) {
            Some(stored) if stored == key_fingerprint => KnownHostResult::Trusted,
            Some(_) => KnownHostResult::Mismatch,
            None => KnownHostResult::Unknown,
        }
    }

    /// Serialize the store to a JSON file at `path`.
    ///
    /// # Errors
    /// Returns [`SshError::IoError`] if the file cannot be created or written.
    pub fn save_to_path(&self, path: &Path) -> Result<(), SshError> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            SshError::IoError(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        })?;
        std::fs::write(path, json).map_err(SshError::from)?;
        Ok(())
    }

    /// Load a known-hosts store from a JSON file at `path`.
    ///
    /// If the file does not exist, an empty store is returned (TOFU bootstrap).
    ///
    /// # Errors
    /// Returns [`SshError::IoError`] if the file exists but cannot be read, or
    /// if the contents are not valid JSON.
    pub fn load_from_path(path: &Path) -> Result<Self, SshError> {
        match std::fs::read_to_string(path) {
            Ok(contents) => {
                let store: Self = serde_json::from_str(&contents).map_err(|e| {
                    SshError::IoError(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
                })?;
                Ok(store)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::new()),
            Err(e) => Err(SshError::from(e)),
        }
    }

    /// Returns the number of entries in the store.
    #[must_use]
    pub fn len(&self) -> usize {
        self.hosts.len()
    }

    /// Returns `true` if the store contains no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.hosts.is_empty()
    }
}

// Re-export `io` for the `save_to_path`/`load_from_path` error conversions.
use std::io;
