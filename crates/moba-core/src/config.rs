//! Session configuration persistence with TOML serialization.
//!
//! This module provides [`SessionEntry`] for representing a single session
//! and [`SessionStore`] for managing a collection of sessions with save/load
//! to disk via TOML files.

use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during configuration save/load operations.
#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)] // variant names are spec'd as *Error
pub enum ConfigError {
    /// An I/O error occurred while reading or writing the config file.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    /// An error occurred while serializing the session store to TOML.
    #[error("Serialization error: {0}")]
    SerializeError(String),
    /// An error occurred while deserializing TOML into a session store.
    #[error("Deserialization error: {0}")]
    DeserializeError(String),
}

/// A single session entry representing a saved connection configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEntry {
    /// Unique identifier for this session.
    pub id: String,
    /// Human-readable name for this session.
    pub name: String,
    /// Type of the session (e.g. "ssh", "telnet", "serial").
    pub session_type: String,
    /// Remote host address, if applicable.
    pub host: Option<String>,
    /// Remote port number, if applicable.
    pub port: Option<u16>,
    /// Username for authentication, if applicable.
    pub username: Option<String>,
    /// User-assigned tags for grouping or filtering.
    pub tags: Vec<String>,
}

/// A collection of [`SessionEntry`] objects with persistence support.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStore {
    /// The list of sessions in this store.
    pub sessions: Vec<SessionEntry>,
}

impl SessionStore {
    /// Create a new empty session store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
        }
    }

    /// Add a session entry to the store.
    pub fn add(&mut self, session: SessionEntry) {
        self.sessions.push(session);
    }

    /// Remove a session by its id. Returns `true` if a session was removed.
    pub fn remove(&mut self, id: &str) -> bool {
        let initial_len = self.sessions.len();
        self.sessions.retain(|s| s.id != id);
        self.sessions.len() != initial_len
    }

    /// Get a reference to a session by its id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&SessionEntry> {
        self.sessions.iter().find(|s| s.id == id)
    }

    /// Get a slice of all sessions in the store.
    #[must_use]
    pub fn list(&self) -> &[SessionEntry] {
        &self.sessions
    }

    /// Find all sessions whose name matches the given string exactly.
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Vec<&SessionEntry> {
        self.sessions.iter().filter(|s| s.name == name).collect()
    }

    /// Serialize the store to a TOML file at the given path.
    ///
    /// # Errors
    /// Returns [`ConfigError::IoError`] if the file cannot be written,
    /// or [`ConfigError::SerializeError`] if TOML serialization fails.
    pub fn save_to_path(&self, path: &Path) -> Result<(), ConfigError> {
        let toml_string =
            toml::to_string_pretty(self).map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        std::fs::write(path, toml_string)?;
        Ok(())
    }

    /// Load a session store from a TOML file at the given path.
    ///
    /// # Errors
    /// Returns [`ConfigError::IoError`] if the file cannot be read,
    /// or [`ConfigError::DeserializeError`] if TOML deserialization fails.
    pub fn load_from_path(path: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        let store: SessionStore =
            toml::from_str(&contents).map_err(|e| ConfigError::DeserializeError(e.to_string()))?;
        Ok(store)
    }

    /// Merge another store into this one, deduplicating by session id.
    ///
    /// Sessions from `other` whose id is not already present in `self` are
    /// appended. Existing sessions with the same id are left unchanged.
    pub fn merge(&mut self, other: SessionStore) {
        for session in other.sessions {
            if !self.sessions.iter().any(|s| s.id == session.id) {
                self.sessions.push(session);
            }
        }
    }
}

// Re-export toml crate so this module compiles standalone (it is used as a
// dev-dependency in the workspace but the module references `toml::` APIs).
extern crate toml;
