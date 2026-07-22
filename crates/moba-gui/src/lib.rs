//! moba-gui library: app shell, terminal view, session management.
//!
//! Re-exports the terminal view widget for use by the binary.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod term_view;

pub use term_view::{TermView, TermViewConfig};
