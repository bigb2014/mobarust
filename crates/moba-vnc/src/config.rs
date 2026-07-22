//! VNC client configuration model.
//!
//! Represents VNC connection parameters used for VNC sessions.

use serde::{Deserialize, Serialize};

/// VNC pixel format descriptor.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelFormat {
    /// Bits per pixel.
    pub bits_per_pixel: u8,
    /// Color depth.
    pub depth: u8,
    /// Big-endian flag.
    pub big_endian: bool,
    /// True color flag.
    pub true_color: bool,
    /// Red maximum.
    pub red_max: u16,
    /// Green maximum.
    pub green_max: u16,
    /// Blue maximum.
    pub blue_max: u16,
    /// Red shift.
    pub red_shift: u8,
    /// Green shift.
    pub green_shift: u8,
    /// Blue shift.
    pub blue_shift: u8,
}

impl Default for PixelFormat {
    fn default() -> Self {
        Self {
            bits_per_pixel: 32,
            depth: 24,
            big_endian: false,
            true_color: true,
            red_max: 255,
            green_max: 255,
            blue_max: 255,
            red_shift: 16,
            green_shift: 8,
            blue_shift: 0,
        }
    }
}

/// VNC connection configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VncConfig {
    /// Remote host.
    pub host: String,
    /// Remote port (default 5900).
    pub port: u16,
    /// VNC password (if any).
    pub password: Option<String>,
    /// Display number (e.g. 0 for port 5900, 1 for port 5901).
    pub display: u16,
    /// Whether to use read-only mode (no input sent).
    pub read_only: bool,
    /// Whether to scale the remote desktop to fit the window.
    pub scale: bool,
    /// Color depth for the connection.
    pub color_depth: u8,
}

impl VncConfig {
    /// Creates a new VNC config for the given host and display number.
    #[must_use]
    pub fn new(host: &str, display: u16) -> Self {
        Self {
            host: host.to_string(),
            port: 5900 + display,
            password: None,
            display,
            read_only: false,
            scale: true,
            color_depth: 24,
        }
    }

    /// Returns the display label.
    #[must_use]
    pub fn display_label(&self) -> String {
        format!("{}:{}", self.host, self.display)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_format_default() {
        let pf = PixelFormat::default();
        assert_eq!(pf.bits_per_pixel, 32);
        assert_eq!(pf.depth, 24);
        assert!(pf.true_color);
    }

    #[test]
    fn vnc_config_new() {
        let cfg = VncConfig::new("192.168.1.1", 0);
        assert_eq!(cfg.host, "192.168.1.1");
        assert_eq!(cfg.port, 5900);
        assert_eq!(cfg.display, 0);
        assert!(!cfg.read_only);
        assert!(cfg.scale);
    }

    #[test]
    fn vnc_config_display_number_to_port() {
        let cfg = VncConfig::new("host", 5);
        assert_eq!(cfg.port, 5905);
    }

    #[test]
    fn vnc_config_display_label() {
        let cfg = VncConfig::new("10.0.0.1", 2);
        assert_eq!(cfg.display_label(), "10.0.0.1:2");
    }

    #[test]
    fn vnc_config_serde_round_trip() {
        let cfg = VncConfig::new("host", 1);
        let json = serde_json::to_string(&cfg).unwrap();
        let back: VncConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg, back);
    }
}
