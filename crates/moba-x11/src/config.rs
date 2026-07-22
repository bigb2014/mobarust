//! X11 forwarding configuration model.
//!
//! Represents X11 display and forwarding parameters used for X11 sessions.

use serde::{Deserialize, Serialize};

/// X11 display identifier (host:display.screen).
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct X11Display {
    /// Host name (empty for local).
    pub host: String,
    /// Display number.
    pub display: u16,
    /// Screen number (usually 0).
    pub screen: u16,
}

impl X11Display {
    /// Parses a DISPLAY environment variable string (e.g. "host:10.0" or ":0").
    #[must_use]
    pub fn parse(display_str: &str) -> Self {
        let (host, rest) = match display_str.split_once(':') {
            Some((h, r)) => (h.to_string(), r),
            None => (String::new(), display_str),
        };
        let (display, screen) = match rest.split_once('.') {
            Some((d, s)) => (d.parse().unwrap_or(0), s.parse().unwrap_or(0)),
            None => (rest.parse().unwrap_or(0), 0),
        };
        Self {
            host,
            display,
            screen,
        }
    }

    /// Returns the DISPLAY environment variable string.
    #[must_use]
    pub fn to_display_string(&self) -> String {
        format!("{}:{}.{}", self.host, self.display, self.screen)
    }

    /// Returns the TCP port for this display (6000 + display).
    #[must_use]
    pub fn port(&self) -> u16 {
        6000 + self.display
    }
}

impl std::fmt::Display for X11Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

/// X11 forwarding configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct X11ForwardConfig {
    /// The local X11 display to forward to.
    pub display: X11Display,
    /// Whether to use X11 forwarding over SSH.
    pub use_ssh_forward: bool,
    /// Authentication cookie (MIT-MAGIC-COOKIE-1).
    pub auth_cookie: Option<String>,
    /// The X authority file path.
    pub xauthority: Option<String>,
}

impl X11ForwardConfig {
    /// Creates a new X11 forwarding config for the default local display.
    #[must_use]
    pub fn new() -> Self {
        Self {
            display: X11Display::parse(":0"),
            use_ssh_forward: true,
            auth_cookie: None,
            xauthority: None,
        }
    }

    /// Creates a config from the DISPLAY and XAUTHORITY environment variables.
    #[must_use]
    pub fn from_env() -> Self {
        let display = std::env::var("DISPLAY")
            .map(|s| X11Display::parse(&s))
            .unwrap_or_else(|_| X11Display::parse(":0"));
        let xauthority = std::env::var("XAUTHORITY").ok();
        Self {
            display,
            use_ssh_forward: true,
            auth_cookie: None,
            xauthority,
        }
    }
}

impl Default for X11ForwardConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_display_local() {
        let d = X11Display::parse(":0");
        assert!(d.host.is_empty());
        assert_eq!(d.display, 0);
        assert_eq!(d.screen, 0);
    }

    #[test]
    fn parse_display_with_screen() {
        let d = X11Display::parse(":10.2");
        assert_eq!(d.display, 10);
        assert_eq!(d.screen, 2);
    }

    #[test]
    fn parse_display_with_host() {
        let d = X11Display::parse("remote:5.0");
        assert_eq!(d.host, "remote");
        assert_eq!(d.display, 5);
    }

    #[test]
    fn parse_display_no_colon() {
        let d = X11Display::parse("0");
        assert!(d.host.is_empty());
        assert_eq!(d.display, 0);
    }

    #[test]
    fn display_to_string() {
        let d = X11Display::parse("host:10.2");
        assert_eq!(d.to_display_string(), "host:10.2");
    }

    #[test]
    fn display_port() {
        let d = X11Display::parse(":10");
        assert_eq!(d.port(), 6010);
    }

    #[test]
    fn x11_config_new() {
        let cfg = X11ForwardConfig::new();
        assert!(cfg.use_ssh_forward);
        assert_eq!(cfg.display.display, 0);
    }

    #[test]
    fn x11_config_serde_round_trip() {
        let cfg = X11ForwardConfig::new();
        let json = serde_json::to_string(&cfg).unwrap();
        let back: X11ForwardConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, back);
    }
}
