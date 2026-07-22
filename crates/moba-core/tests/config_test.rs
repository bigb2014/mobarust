//! Tests for the TOML-backed session store (M2-T02).
//!
//! These tests include `config.rs` via `#[path]` so they run independently
//! of whatever `session.rs` the other subagent is building in parallel.

#[path = "../src/config.rs"]
mod config;

use std::env::temp_dir;
use std::fs;
use std::path::PathBuf;

use config::{ConfigError, SessionEntry, SessionStore};

/// Helper to build a sample session entry.
fn make_entry(id: &str, name: &str, session_type: &str) -> SessionEntry {
    SessionEntry {
        id: id.to_string(),
        name: name.to_string(),
        session_type: session_type.to_string(),
        host: Some("example.com".to_string()),
        port: Some(22),
        username: Some("user".to_string()),
        tags: vec!["prod".to_string(), "ssh".to_string()],
    }
}

/// Helper to build a unique temp file path.
fn temp_file(name: &str) -> PathBuf {
    let mut path = temp_dir();
    path.push(format!(
        "moba_core_config_test_{}_{}.toml",
        name,
        std::process::id()
    ));
    path
}

#[test]
fn new_store_is_empty() {
    let store = SessionStore::new();
    assert!(store.list().is_empty());
}

#[test]
fn add_and_get_session() {
    let mut store = SessionStore::new();
    let entry = make_entry("s1", "web-server", "ssh");
    store.add(entry);

    assert_eq!(store.list().len(), 1);
    let got = store.get("s1");
    assert!(got.is_some());
    assert_eq!(got.unwrap().name, "web-server");
    assert!(store.get("nonexistent").is_none());
}

#[test]
fn remove_session() {
    let mut store = SessionStore::new();
    store.add(make_entry("s1", "alpha", "ssh"));
    store.add(make_entry("s2", "beta", "ssh"));

    assert!(store.remove("s1"));
    assert_eq!(store.list().len(), 1);
    assert!(!store.remove("s1"));
    assert_eq!(store.list().len(), 1);
}

#[test]
fn find_by_name() {
    let mut store = SessionStore::new();
    store.add(make_entry("s1", "alpha", "ssh"));
    store.add(make_entry("s2", "alpha", "telnet"));
    store.add(make_entry("s3", "beta", "ssh"));

    let found = store.find_by_name("alpha");
    assert_eq!(found.len(), 2);
    assert_eq!(store.find_by_name("gamma").len(), 0);
}

#[test]
fn save_and_load_round_trip() {
    let mut store = SessionStore::new();
    store.add(make_entry("s1", "alpha", "ssh"));
    store.add(make_entry("s2", "beta", "telnet"));

    let path = temp_file("round_trip");
    // Clean up from any previous run.
    let _ = fs::remove_file(&path);

    store.save_to_path(&path).expect("save should succeed");

    let loaded = SessionStore::load_from_path(&path).expect("load should succeed");

    assert_eq!(store.list().len(), loaded.list().len());
    assert_eq!(store.list()[0], loaded.list()[0]);
    assert_eq!(store.list()[1], loaded.list()[1]);

    let _ = fs::remove_file(&path);
}

#[test]
fn merge_stores() {
    let mut store_a = SessionStore::new();
    store_a.add(make_entry("s1", "alpha", "ssh"));
    store_a.add(make_entry("s2", "beta", "ssh"));

    let mut store_b = SessionStore::new();
    store_b.add(make_entry("s2", "beta-dup", "ssh")); // duplicate id
    store_b.add(make_entry("s3", "gamma", "telnet"));

    store_a.merge(store_b);

    assert_eq!(store_a.list().len(), 3);
    // Original s2 should be unchanged (not overwritten by store_b's s2).
    assert_eq!(store_a.get("s2").unwrap().name, "beta");
    assert!(store_a.get("s3").is_some());
}

#[test]
fn save_to_nonexistent_dir_returns_error() {
    let store = SessionStore::new();
    let path = PathBuf::from("/this/path/definitely/does/not/exist/config.toml");

    let result = store.save_to_path(&path);
    assert!(result.is_err());
    match result {
        Err(ConfigError::IoError(_)) => {}
        Err(other) => panic!("expected IoError, got {other:?}"),
        Ok(_) => panic!("expected error, got Ok"),
    }
}
