//! SSH tunnel manager: local, remote, and dynamic (SOCKS) port forwarding.
//!
//! Manages port forwarding rules that can be applied to an SSH connection.
//! Each rule specifies a direction, local/remote bind address and port,
//! and a target host and port on the remote/local side.

use serde::{Deserialize, Serialize};

/// The direction of a port forward.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ForwardType {
    /// Local port forward: localhost:local_port -> remote_host:remote_port via SSH.
    Local,
    /// Remote port forward: remote_bind:remote_port -> target_host:target_port via SSH.
    Remote,
    /// Dynamic SOCKS proxy: localhost:local_port acts as a SOCKS5 proxy tunneling through SSH.
    Dynamic,
}

/// A port forwarding rule.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TunnelRule {
    /// Unique identifier for this rule.
    pub id: String,
    /// Forward direction.
    pub forward_type: ForwardType,
    /// Local bind address (e.g. "127.0.0.1" or "0.0.0.0").
    pub local_addr: String,
    /// Local port to listen on (for Local/Dynamic) or connect to (for Remote).
    pub local_port: u16,
    /// Remote bind address (for Remote) or target host (for Local).
    pub remote_addr: String,
    /// Remote port (for Remote) or target port (for Local).
    pub remote_port: u16,
    /// Whether this tunnel is currently active.
    pub active: bool,
}

impl TunnelRule {
    /// Creates a new local port forward rule.
    #[must_use]
    pub fn local(id: &str, local_port: u16, remote_host: &str, remote_port: u16) -> Self {
        Self {
            id: id.to_string(),
            forward_type: ForwardType::Local,
            local_addr: "127.0.0.1".to_string(),
            local_port,
            remote_addr: remote_host.to_string(),
            remote_port,
            active: false,
        }
    }

    /// Creates a new remote port forward rule.
    #[must_use]
    pub fn remote(id: &str, remote_port: u16, target_host: &str, target_port: u16) -> Self {
        Self {
            id: id.to_string(),
            forward_type: ForwardType::Remote,
            local_addr: "127.0.0.1".to_string(),
            local_port: target_port,
            remote_addr: target_host.to_string(),
            remote_port,
            active: false,
        }
    }

    /// Creates a new dynamic (SOCKS) forward rule.
    #[must_use]
    pub fn dynamic(id: &str, local_port: u16) -> Self {
        Self {
            id: id.to_string(),
            forward_type: ForwardType::Dynamic,
            local_addr: "127.0.0.1".to_string(),
            local_port,
            remote_addr: String::new(),
            remote_port: 0,
            active: false,
        }
    }

    /// Returns a human-readable description of the tunnel.
    #[must_use]
    pub fn description(&self) -> String {
        match self.forward_type {
            ForwardType::Local => {
                format!(
                    "L {}:{} -> {}:{}",
                    self.local_addr, self.local_port, self.remote_addr, self.remote_port
                )
            }
            ForwardType::Remote => {
                format!(
                    "R {}:{} -> {}:{}",
                    self.local_addr, self.remote_port, self.remote_addr, self.local_port
                )
            }
            ForwardType::Dynamic => {
                format!("D {}:{} (SOCKS)", self.local_addr, self.local_port)
            }
        }
    }
}

/// The tunnel manager: holds a collection of forwarding rules.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TunnelManager {
    rules: Vec<TunnelRule>,
}

impl TunnelManager {
    /// Creates a new empty tunnel manager.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a tunnel rule.
    pub fn add(&mut self, rule: TunnelRule) {
        self.rules.push(rule);
    }

    /// Removes a tunnel rule by id.
    pub fn remove(&mut self, id: &str) -> bool {
        let len = self.rules.len();
        self.rules.retain(|r| r.id != id);
        self.rules.len() < len
    }

    /// Returns a rule by id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&TunnelRule> {
        self.rules.iter().find(|r| r.id == id)
    }

    /// Returns all rules.
    #[must_use]
    pub fn rules(&self) -> &[TunnelRule] {
        &self.rules
    }

    /// Returns mutable access to all rules.
    pub fn rules_mut(&mut self) -> &mut [TunnelRule] {
        &mut self.rules
    }

    /// Activates a tunnel rule by id.
    pub fn activate(&mut self, id: &str) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == id) {
            rule.active = true;
            true
        } else {
            false
        }
    }

    /// Deactivates a tunnel rule by id.
    pub fn deactivate(&mut self, id: &str) -> bool {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == id) {
            rule.active = false;
            true
        } else {
            false
        }
    }

    /// Returns only active rules.
    #[must_use]
    pub fn active_rules(&self) -> Vec<&TunnelRule> {
        self.rules.iter().filter(|r| r.active).collect()
    }

    /// Returns only inactive rules.
    #[must_use]
    pub fn inactive_rules(&self) -> Vec<&TunnelRule> {
        self.rules.iter().filter(|r| !r.active).collect()
    }

    /// Clears all rules.
    pub fn clear(&mut self) {
        self.rules.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_tunnel_rule() {
        let r = TunnelRule::local("t1", 8080, "internal.example.com", 80);
        assert_eq!(r.forward_type, ForwardType::Local);
        assert_eq!(r.local_port, 8080);
        assert_eq!(r.remote_addr, "internal.example.com");
        assert_eq!(r.remote_port, 80);
        assert!(!r.active);
    }

    #[test]
    fn remote_tunnel_rule() {
        let r = TunnelRule::remote("t2", 9090, "target.example.com", 443);
        assert_eq!(r.forward_type, ForwardType::Remote);
        assert_eq!(r.remote_port, 9090);
    }

    #[test]
    fn dynamic_tunnel_rule() {
        let r = TunnelRule::dynamic("t3", 1080);
        assert_eq!(r.forward_type, ForwardType::Dynamic);
        assert_eq!(r.local_port, 1080);
        assert!(r.remote_addr.is_empty());
    }

    #[test]
    fn description_formats() {
        let l = TunnelRule::local("t", 8080, "host", 80);
        assert!(l.description().contains("L 127.0.0.1:8080"));
        let r = TunnelRule::remote("t", 9090, "host", 443);
        assert!(r.description().contains("R"));
        let d = TunnelRule::dynamic("t", 1080);
        assert!(d.description().contains("SOCKS"));
    }

    #[test]
    fn manager_add_and_get() {
        let mut mgr = TunnelManager::new();
        mgr.add(TunnelRule::local("t1", 8080, "host", 80));
        assert_eq!(mgr.rules().len(), 1);
        assert!(mgr.get("t1").is_some());
        assert!(mgr.get("t2").is_none());
    }

    #[test]
    fn manager_remove() {
        let mut mgr = TunnelManager::new();
        mgr.add(TunnelRule::local("t1", 8080, "host", 80));
        mgr.add(TunnelRule::dynamic("t2", 1080));
        assert!(mgr.remove("t1"));
        assert_eq!(mgr.rules().len(), 1);
        assert!(!mgr.remove("nonexistent"));
    }

    #[test]
    fn manager_activate_deactivate() {
        let mut mgr = TunnelManager::new();
        mgr.add(TunnelRule::local("t1", 8080, "host", 80));
        assert!(mgr.activate("t1"));
        assert!(mgr.get("t1").is_some_and(|r| r.active));
        assert_eq!(mgr.active_rules().len(), 1);
        assert!(mgr.deactivate("t1"));
        assert!(!mgr.get("t1").is_some_and(|r| r.active));
        assert_eq!(mgr.active_rules().len(), 0);
    }

    #[test]
    fn manager_active_inactive() {
        let mut mgr = TunnelManager::new();
        mgr.add(TunnelRule::local("t1", 80, "h", 80));
        mgr.add(TunnelRule::local("t2", 81, "h", 81));
        mgr.activate("t1");
        assert_eq!(mgr.active_rules().len(), 1);
        assert_eq!(mgr.inactive_rules().len(), 1);
    }

    #[test]
    fn manager_clear() {
        let mut mgr = TunnelManager::new();
        mgr.add(TunnelRule::local("t1", 80, "h", 80));
        mgr.add(TunnelRule::dynamic("t2", 1080));
        mgr.clear();
        assert!(mgr.rules().is_empty());
    }

    #[test]
    fn serde_round_trip() {
        let mut mgr = TunnelManager::new();
        mgr.add(TunnelRule::local("t1", 8080, "host", 80));
        mgr.add(TunnelRule::dynamic("t2", 1080));
        let json = serde_json::to_string(&mgr).unwrap();
        let back: TunnelManager = serde_json::from_str(&json).unwrap();
        assert_eq!(mgr.rules().len(), back.rules().len());
        assert_eq!(mgr.rules()[0], back.rules()[0]);
    }

    #[test]
    fn rule_serde_round_trip() {
        let r = TunnelRule::remote("t1", 9090, "host", 443);
        let json = serde_json::to_string(&r).unwrap();
        let back: TunnelRule = serde_json::from_str(&json).unwrap();
        assert_eq!(r, back);
    }
}
