//! RDP client configuration model.
//!
//! Represents RDP connection parameters used for RDP sessions.

use serde::{Deserialize, Serialize};

/// RDP color depth options.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorDepth {
    /// 8-bit color.
    Bits8,
    /// 15-bit color.
    Bits15,
    /// 16-bit color.
    Bits16,
    /// 24-bit color.
    Bits24,
    /// 32-bit color.
    Bits32,
}

impl ColorDepth {
    /// Returns the bits value.
    #[must_use]
    pub fn bits(&self) -> u8 {
        match self {
            Self::Bits8 => 8,
            Self::Bits15 => 15,
            Self::Bits16 => 16,
            Self::Bits24 => 24,
            Self::Bits32 => 32,
        }
    }
}

/// RDP authentication method.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Password authentication.
    Password,
    /// Network Level Authentication (NLA).
    Nla,
    /// Smart card authentication.
    SmartCard,
}

/// RDP connection configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RdpConfig {
    /// Remote host.
    pub host: String,
    /// Remote port (default 3389).
    pub port: u16,
    /// Username.
    pub username: Option<String>,
    /// Domain (for AD-joined hosts).
    pub domain: Option<String>,
    /// Color depth.
    pub color_depth: ColorDepth,
    /// Authentication method.
    pub auth_method: AuthMethod,
    /// Screen width in pixels.
    pub width: u16,
    /// Screen height in pixels.
    pub height: u16,
    /// Whether to enable keyboard redirection.
    pub enable_keyboard: bool,
    /// Whether to enable clipboard redirection.
    pub enable_clipboard: bool,
    /// Whether to enable audio redirection.
    pub enable_audio: bool,
}

impl RdpConfig {
    /// Creates a new RDP config for the given host with default settings.
    #[must_use]
    pub fn new(host: &str) -> Self {
        Self {
            host: host.to_string(),
            port: 3389,
            username: None,
            domain: None,
            color_depth: ColorDepth::Bits32,
            auth_method: AuthMethod::Nla,
            width: 1920,
            height: 1080,
            enable_keyboard: true,
            enable_clipboard: true,
            enable_audio: false,
        }
    }

    /// Returns the display label.
    #[must_use]
    pub fn display_label(&self) -> String {
        let user = self.username.as_deref().unwrap_or("");
        let dom = self.domain.as_deref().unwrap_or("");
        if user.is_empty() {
            format!("{}:{}", self.host, self.port)
        } else if dom.is_empty() {
            format!("{}@{}:{}", user, self.host, self.port)
        } else {
            format!("{}\\{}@{}:{}", dom, user, self.host, self.port)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_depth_bits() {
        assert_eq!(ColorDepth::Bits8.bits(), 8);
        assert_eq!(ColorDepth::Bits16.bits(), 16);
        assert_eq!(ColorDepth::Bits32.bits(), 32);
    }

    #[test]
    fn rdp_config_new() {
        let cfg = RdpConfig::new("192.168.1.100");
        assert_eq!(cfg.host, "192.168.1.100");
        assert_eq!(cfg.port, 3389);
        assert_eq!(cfg.color_depth, ColorDepth::Bits32);
        assert_eq!(cfg.auth_method, AuthMethod::Nla);
        assert_eq!(cfg.width, 1920);
        assert_eq!(cfg.height, 1080);
    }

    #[test]
    fn rdp_config_display_label_no_user() {
        let cfg = RdpConfig::new("10.0.0.1");
        assert_eq!(cfg.display_label(), "10.0.0.1:3389");
    }

    #[test]
    fn rdp_config_display_label_with_user() {
        let mut cfg = RdpConfig::new("10.0.0.1");
        cfg.username = Some("admin".to_string());
        assert_eq!(cfg.display_label(), "admin@10.0.0.1:3389");
    }

    #[test]
    fn rdp_config_display_label_with_domain() {
        let mut cfg = RdpConfig::new("10.0.0.1");
        cfg.username = Some("admin".to_string());
        cfg.domain = Some("CORP".to_string());
        assert_eq!(cfg.display_label(), "CORP\\admin@10.0.0.1:3389");
    }

    #[test]
    fn rdp_config_serde_round_trip() {
        let mut cfg = RdpConfig::new("host");
        cfg.username = Some("user".to_string());
        cfg.color_depth = ColorDepth::Bits16;
        let json = serde_json::to_string(&cfg).unwrap();
        let back: RdpConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, back);
    }
}
