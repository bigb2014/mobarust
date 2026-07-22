//! Scrollback buffer for terminal line history.
//!
//! Stores lines that have scrolled off the top of the visible grid.
//! Lines are stored most-recent-first: index 0 is the most recently
//! scrolled-off line, and higher indices are older.
//!
//! The buffer has a configurable maximum capacity. When the number of
//! stored lines exceeds the capacity, the oldest lines are trimmed.

use serde::{Deserialize, Serialize};

/// Text attributes for a styled character cell.
///
/// This is a minimal style representation used by the scrollback buffer.
/// It will be reconciled with the full grid style type by the orchestrator.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Style {
    /// Foreground color index (0 = default).
    pub fg: u8,
    /// Background color index (0 = default).
    pub bg: u8,
    /// Bitmask of text attributes (bold, italic, underline, etc.).
    pub attrs: u8,
}

/// A single terminal line: a sequence of styled characters.
pub type Line = Vec<(char, Style)>;

/// Default maximum number of lines in the scrollback buffer.
const DEFAULT_CAPACITY: usize = 10_000;

/// A scrollback buffer that stores lines scrolled off the visible grid.
///
/// Lines are stored in most-recent-first order: `get(0)` returns the
/// most recently scrolled-off line. When the buffer exceeds its capacity,
/// the oldest lines are silently trimmed.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scrollback {
    /// Stored lines, most-recent-first.
    lines: Vec<Line>,
    /// Maximum number of lines to retain.
    capacity: usize,
    /// Total number of lines ever pushed (including trimmed ones).
    total_pushed: u64,
}

impl Scrollback {
    /// Creates a new scrollback buffer with the default capacity (10 000).
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Creates a new scrollback buffer with the given maximum capacity.
    ///
    /// # Panics
    /// This function does not panic; a capacity of 0 means no lines are
    /// ever retained.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            lines: Vec::new(),
            capacity,
            total_pushed: 0,
        }
    }

    /// Pushes a line onto the scrollback buffer.
    ///
    /// The line becomes the most recently scrolled-off line (index 0).
    /// If the buffer is at capacity, the oldest line is trimmed.
    pub fn push(&mut self, line: Line) {
        if self.capacity == 0 {
            self.total_pushed += 1;
            return;
        }
        self.lines.insert(0, line);
        self.total_pushed += 1;
        if self.lines.len() > self.capacity {
            self.lines.truncate(self.capacity);
        }
    }

    /// Returns a reference to the line at the given index, or `None` if
    /// the index is out of bounds.
    ///
    /// Index 0 is the most recently scrolled-off line.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Line> {
        self.lines.get(index)
    }

    /// Returns the number of lines currently stored in the buffer.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Returns `true` if the buffer contains no lines.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Returns the maximum number of lines the buffer will retain.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the total number of lines ever pushed, including trimmed ones.
    #[must_use]
    pub fn total_pushed(&self) -> u64 {
        self.total_pushed
    }

    /// Removes all lines from the buffer, preserving the capacity.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Returns an iterator over the stored lines, most-recent-first.
    pub fn iter(&self) -> impl Iterator<Item = &Line> {
        self.lines.iter()
    }
}

impl Default for Scrollback {
    fn default() -> Self {
        Self::new()
    }
}
