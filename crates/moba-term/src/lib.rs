//! moba-term crate.
//!
//! Terminal engine: PTY, VT parsing, grid, scrollback, selection.
//!
//! See `docs/TASKS.md` for the current task ledger and `docs/PARITY.md`
//! for the feature-parity matrix.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod grid;
pub mod scrollback;
pub mod selection;
pub mod vt_parser;

pub use grid::{Attributes, Cell, ClearMode, Color, Cursor, Grid};
pub use scrollback::{Line, Scrollback, Style};
pub use selection::{Position, Selection, SelectionMode, TextGrid};
pub use vt_parser::Terminal;
