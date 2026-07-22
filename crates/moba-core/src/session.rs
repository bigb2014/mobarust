//! Session configuration model for saved terminal sessions.
//!
//! This module defines the domain types used by the session manager and the
//! sidebar to represent persisted sessions (SSH, local shell, telnet, serial,
//! etc.) and groups thereof.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Monotonic counter used to guarantee unique session ids even when two
/// sessions are created within the same millisecond.
static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// The kind of terminal or remote session a [`SessionConfig`] represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SessionType {
    /// A local shell spawned on the user's machine.
    LocalShell,
    /// An SSH connection to a remote host.
    Ssh,
    /// A telnet connection to a remote host.
    Telnet,
    /// An RDP (Remote Desktop Protocol) session.
    Rdp,
    /// A VNC (Virtual Network Computing) session.
    Vnc,
    /// A serial port connection.
    Serial,
    /// A Mosh (Mobile Shell) connection.
    Mosh,
    /// An FTP file-transfer session.
    Ftp,
    /// An SFTP file-transfer session.
    Sftp,
}

impl SessionType {
    /// Returns a human-friendly label for the variant, used in display
    /// strings and sidebar UI.
    #[must_use]
    fn label(self) -> &'static str {
        match self {
            Self::LocalShell => "LocalShell",
            Self::Ssh => "Ssh",
            Self::Telnet => "Telnet",
            Self::Rdp => "Rdp",
            Self::Vnc => "Vnc",
            Self::Serial => "Serial",
            Self::Mosh => "Mosh",
            Self::Ftp => "Ftp",
            Self::Sftp => "Sftp",
        }
    }
}

/// A persisted session configuration describing how to launch a terminal or
/// remote connection.
///
/// Instances are created via [`SessionConfig::new`] and then optionally
/// mutated to fill in connection details (host, port, username, ...).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Unique identifier for the session (timestamp + counter based).
    pub id: String,
    /// Human-readable display name, e.g. `"Production Server"`.
    pub name: String,
    /// The protocol/kind of this session.
    pub session_type: SessionType,
    /// Remote host address; `None` for local sessions.
    pub host: Option<String>,
    /// Remote port; `None` for local sessions or protocol defaults.
    pub port: Option<u16>,
    /// Username for remote authentication; `None` if not applicable.
    pub username: Option<String>,
    /// Custom command to execute instead of the default shell.
    pub command: Option<String>,
    /// Initial working directory for the session.
    pub working_dir: Option<String>,
    /// Free-form tags for grouping and filtering in the sidebar.
    pub tags: Vec<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
    /// ISO 8601 last-updated timestamp.
    pub updated_at: String,
}

impl SessionConfig {
    /// Creates a new `SessionConfig` with the given display name and type,
    /// generating a unique id and current timestamps.
    ///
    /// Connection fields (`host`, `port`, `username`, `command`,
    /// `working_dir`) start as `None`/empty and should be set by the caller
    /// after construction.
    #[must_use]
    pub fn new(name: &str, session_type: SessionType) -> Self {
        let now = iso8601_now();
        let seq = ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let id = format!("{now}-{seq}");
        Self {
            id,
            name: name.to_string(),
            session_type,
            host: None,
            port: None,
            username: None,
            command: None,
            working_dir: None,
            tags: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Returns a display label for the session.
    ///
    /// For `LocalShell` the label is just the name; for all other types the
    /// label is `"name (Type)"`.
    #[must_use]
    pub fn display_label(&self) -> String {
        match self.session_type {
            SessionType::LocalShell => self.name.clone(),
            other => format!("{} ({})", self.name, other.label()),
        }
    }
}

/// A named group of sessions, used to organise the sidebar tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionGroup {
    /// Human-readable group name, e.g. `"Dev Environments"`.
    pub name: String,
    /// Sessions belonging to this group, in display order.
    pub sessions: Vec<SessionConfig>,
}

/// Returns the current time as an ISO 8601 string (UTC, second precision).
///
/// We avoid pulling in a full `chrono` dependency: the timestamp is built
/// manually from `SystemTime` and formatted as `YYYYMMDDTHHMMSSZ`, which is a
/// valid (compact) ISO 8601 representation.
fn iso8601_now() -> String {
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    let (year, month, day, hour, minute, second) = epoch_to_calendar(secs);
    format!("{year:04}{month:02}{day:02}T{hour:02}{minute:02}{second:02}Z")
}

/// Converts a Unix epoch seconds value into UTC calendar components.
///
/// This is a minimal civil-from-days implementation (Howard Hinnant's
/// algorithm) -- no external date crate required.
///
/// # Returns
/// `(year, month, day, hour, minute, second)` all as `i32`.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn epoch_to_calendar(secs: u64) -> (i32, i32, i32, i32, i32, i32) {
    let days = (secs / 86_400) as i64;
    let rem = (secs % 86_400) as i64;
    let hour = (rem / 3600) as i32;
    let minute = ((rem % 3600) / 60) as i32;
    let second = (rem % 60) as i32;

    // Howard Hinnant's civil-from-days algorithm.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as i32; // [1, 31]
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as i32; // [1, 12]
    let year = (y + i64::from(m <= 2)) as i32;

    (year, m, d, hour, minute, second)
}
