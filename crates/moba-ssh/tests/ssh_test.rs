//! Unit tests for `moba-ssh`: known_hosts logic, error display, and serialization.
//!
//! These tests intentionally avoid real SSH connections; those require the
//! docker sshd test bed and are covered by E2E tests (`--features e2e`).

use moba_ssh::{KnownHostResult, KnownHosts, SshError};
use std::io::Write;

/// A newly created `KnownHosts` store verifies no hosts.
#[test]
fn known_hosts_new_is_empty() {
    let kh = KnownHosts::new();
    assert_eq!(
        kh.verify("example.com", "fingerprint-A"),
        KnownHostResult::Unknown
    );
}

/// After adding a host+fingerprint, the same pair verifies as Trusted.
#[test]
fn known_hosts_add_and_verify_trusted() {
    let mut kh = KnownHosts::new();
    kh.add("example.com", "fingerprint-A");
    assert_eq!(
        kh.verify("example.com", "fingerprint-A"),
        KnownHostResult::Trusted
    );
}

/// A fingerprint that does not match the stored one yields Mismatch.
#[test]
fn known_hosts_verify_mismatch_returns_mismatch() {
    let mut kh = KnownHosts::new();
    kh.add("example.com", "fingerprint-A");
    assert_eq!(
        kh.verify("example.com", "fingerprint-B"),
        KnownHostResult::Mismatch
    );
}

/// A host that was never added yields Unknown.
#[test]
fn known_hosts_verify_unknown_returns_unknown() {
    let mut kh = KnownHosts::new();
    kh.add("example.com", "fingerprint-A");
    assert_eq!(
        kh.verify("other.example.com", "fingerprint-A"),
        KnownHostResult::Unknown
    );
}

/// Saving a `KnownHosts` store to a JSON file and loading it back preserves
/// all entries (serde round-trip).
#[test]
fn known_hosts_save_load_round_trip() {
    let tmp = tempfile_path();
    let mut kh = KnownHosts::new();
    kh.add("host-a.example.com", "fp-a");
    kh.add("host-b.example.com", "fp-b");

    kh.save_to_path(std::path::Path::new(&tmp))
        .expect("save should succeed");

    let loaded =
        KnownHosts::load_from_path(std::path::Path::new(&tmp)).expect("load should succeed");
    assert_eq!(
        loaded.verify("host-a.example.com", "fp-a"),
        KnownHostResult::Trusted
    );
    assert_eq!(
        loaded.verify("host-b.example.com", "fp-b"),
        KnownHostResult::Trusted
    );
    assert_eq!(
        loaded.verify("host-a.example.com", "fp-b"),
        KnownHostResult::Mismatch
    );
    assert_eq!(
        loaded.verify("host-c.example.com", "fp-a"),
        KnownHostResult::Unknown
    );

    std::fs::remove_file(&tmp).ok();
}

/// Every `SshError` variant renders a non-empty, human-readable message.
#[test]
fn ssh_error_display() {
    let cases: Vec<SshError> = vec![
        SshError::ConnectError("connection refused".to_string()),
        SshError::AuthError("bad password".to_string()),
        SshError::ChannelError("no session".to_string()),
        SshError::HostKeyError("fingerprint mismatch".to_string()),
        SshError::KeyLoadError("file not found".to_string()),
        SshError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "missing file",
        )),
    ];
    for err in &cases {
        let msg = format!("{err}");
        assert!(!msg.is_empty(), "error display must not be empty: {err:?}");
        // Every message must contain at least one ASCII letter.
        assert!(
            msg.chars().any(|c| c.is_ascii_alphabetic()),
            "error display must be human-readable: {err:?}"
        );
    }
}

/// Helper: create a unique temporary file path in the system temp dir.
fn tempfile_path() -> String {
    let mut path = std::env::temp_dir();
    let pid = std::process::id();
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    path.push(format!("moba-ssh-test-{pid}-{ts}.json"));
    path.to_string_lossy().into_owned()
}

/// Sanity: serde_json can (de)serialize a `KnownHosts` store directly.
#[test]
fn known_hosts_serde_json_round_trip() {
    let mut kh = KnownHosts::new();
    kh.add("serde.example.com", "fp-serde");

    let json = serde_json::to_string(&kh).expect("serialize");
    let back: KnownHosts = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        back.verify("serde.example.com", "fp-serde"),
        KnownHostResult::Trusted
    );
}

/// `KnownHostResult` is Clone + PartialEq + Debug, as required by callers.
#[test]
fn known_host_result_traits() {
    let a = KnownHostResult::Trusted;
    let b = a;
    assert_eq!(a, b);
    let _ = format!("{a:?}");
}

/// Smoke-test that `SshClient::new` stores connection parameters without
/// attempting a network connection.
#[test]
fn ssh_client_new_stores_params() {
    use moba_ssh::SshClient;
    let client = SshClient::new("example.com", 2222, "alice");
    // Public accessors confirm the values were stored.
    assert_eq!(client.host(), "example.com");
    assert_eq!(client.port(), 2222);
    assert_eq!(client.username(), "alice");
}

/// Guard against accidental debug prints in library source.
#[test]
fn no_debug_prints_in_lib() {
    let src = std::fs::read_to_string("src/lib.rs").expect("lib.rs readable");
    let modules = ["src/error.rs", "src/known_hosts.rs", "src/client.rs"];
    let mut all = src;
    for m in modules {
        all.push('\n');
        all.push_str(&std::fs::read_to_string(m).expect("module readable"));
    }
    assert!(!all.contains("println!"), "no println! in moba-ssh lib");
    assert!(!all.contains("eprintln!"), "no eprintln! in moba-ssh lib");
    assert!(!all.contains("dbg!"), "no dbg! in moba-ssh lib");
}

/// The `std::io::Write` import is used by the tempfile write helper above.
#[allow(dead_code)]
fn _ensure_write_used(w: &mut dyn Write) {
    let _ = w.write_all(b"");
}
