//! moba-gui library: app shell, terminal view, session management.
//!
//! Re-exports the terminal view widget and app for use by the binary.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod app;
pub mod tabs;
pub mod term_view;

pub use app::MobaApp;
pub use tabs::{TabManager, TerminalTab};
pub use term_view::{TermView, TermViewConfig};
