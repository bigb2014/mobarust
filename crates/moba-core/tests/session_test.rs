//! Integration tests for the session config model.

use moba_core::session::*;

/// A local shell session should have sensible defaults: no host, no port,
/// no username, and a `LocalShell` type.
#[test]
fn create_local_session() {
    let s = SessionConfig::new("Local Terminal", SessionType::LocalShell);
    assert_eq!(s.name, "Local Terminal");
    assert_eq!(s.session_type, SessionType::LocalShell);
    assert!(s.host.is_none(), "local session should have no host");
    assert!(s.port.is_none(), "local session should have no port");
    assert!(
        s.username.is_none(),
        "local session should have no username"
    );
    assert!(
        s.command.is_none(),
        "local session should have no custom command"
    );
    assert!(
        s.working_dir.is_none(),
        "local session should have no working dir"
    );
    assert!(s.tags.is_empty(), "local session should have no tags");
    assert!(!s.id.is_empty(), "session id must not be empty");
    assert!(!s.created_at.is_empty(), "created_at must not be empty");
    assert_eq!(
        s.created_at, s.updated_at,
        "new session timestamps must match"
    );
}

/// An SSH session should retain host/port/username once set via the builder
/// pattern after construction.
#[test]
fn create_ssh_session() {
    let mut s = SessionConfig::new("Production Server", SessionType::Ssh);
    s.host = Some("prod.example.com".to_string());
    s.port = Some(22);
    s.username = Some("admin".to_string());

    assert_eq!(s.session_type, SessionType::Ssh);
    assert_eq!(s.host.as_deref(), Some("prod.example.com"));
    assert_eq!(s.port, Some(22));
    assert_eq!(s.username.as_deref(), Some("admin"));
}

/// Serializing a `SessionConfig` to JSON and deserializing it back must yield
/// an equal value (round-trip).
#[test]
fn serde_round_trip() {
    let mut s = SessionConfig::new("Jump Box", SessionType::Mosh);
    s.host = Some("10.0.0.5".to_string());
    s.port = Some(60000);
    s.username = Some("ops".to_string());
    s.tags = vec!["infra".to_string(), "prod".to_string()];

    let json = serde_json::to_string(&s).expect("serialize must succeed");
    let back: SessionConfig = serde_json::from_str(&json).expect("deserialize must succeed");
    assert_eq!(s, back, "round-trip must preserve equality");
}

/// A `SessionGroup` should serialize and deserialize without loss, and its
/// sessions vector should reflect the order in which sessions were added.
#[test]
fn session_group() {
    let local = SessionConfig::new("Local", SessionType::LocalShell);
    let ssh = SessionConfig::new("Prod", SessionType::Ssh);
    let group = SessionGroup {
        name: "Dev Environments".to_string(),
        sessions: vec![local.clone(), ssh.clone()],
    };

    assert_eq!(group.name, "Dev Environments");
    assert_eq!(group.sessions.len(), 2);
    assert_eq!(group.sessions[0].name, "Local");
    assert_eq!(group.sessions[1].name, "Prod");

    let json = serde_json::to_string(&group).expect("serialize must succeed");
    let back: SessionGroup = serde_json::from_str(&json).expect("deserialize must succeed");
    assert_eq!(group, back, "group round-trip must preserve equality");
}

/// `display_label` should append the session type in parentheses unless the
/// type is `LocalShell`, in which case it returns just the name.
#[test]
fn display_label() {
    let local = SessionConfig::new("Local Terminal", SessionType::LocalShell);
    assert_eq!(local.display_label(), "Local Terminal");

    let ssh = SessionConfig::new("Production Server", SessionType::Ssh);
    assert_eq!(ssh.display_label(), "Production Server (Ssh)");

    let telnet = SessionConfig::new("Legacy Router", SessionType::Telnet);
    assert_eq!(telnet.display_label(), "Legacy Router (Telnet)");

    let rdp = SessionConfig::new("Windows Box", SessionType::Rdp);
    assert_eq!(rdp.display_label(), "Windows Box (Rdp)");

    let sftp = SessionConfig::new("File Server", SessionType::Sftp);
    assert_eq!(sftp.display_label(), "File Server (Sftp)");
}

/// Two sessions created in quick succession should have distinct ids.
#[test]
fn unique_ids() {
    let a = SessionConfig::new("A", SessionType::LocalShell);
    let b = SessionConfig::new("B", SessionType::LocalShell);
    assert_ne!(a.id, b.id, "each new session should get a unique id");
}

/// Every `SessionType` variant should serialize and deserialize correctly.
#[test]
fn session_type_serde() {
    let variants = [
        SessionType::LocalShell,
        SessionType::Ssh,
        SessionType::Telnet,
        SessionType::Rdp,
        SessionType::Vnc,
        SessionType::Serial,
        SessionType::Mosh,
        SessionType::Ftp,
        SessionType::Sftp,
    ];
    for v in variants {
        let json = serde_json::to_string(&v).expect("serialize variant");
        let back: SessionType = serde_json::from_str(&json).expect("deserialize variant");
        assert_eq!(v, back, "variant round-trip failed for {json}");
    }
}
