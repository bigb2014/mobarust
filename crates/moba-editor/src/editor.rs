//! Core text editor backend: buffer, cursor, and editing primitives.
//!
//! The [`TextBuffer`] owns a vector of lines and a [`Cursor`] that tracks the
//! caret position. All editing operations (insert, backspace, delete, cursor
//! movement) are expressed in terms of that cursor, making the buffer suitable
//! as the model behind a modal or GUI editor for remote files.

use std::fmt;

use thiserror::Error;

/// Errors that can arise during editor operations.
#[derive(Debug, Error)]
pub enum EditorError {
    /// Wraps a [`std::io::Error`] from file I/O.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// The requested (row, column) position is outside the buffer bounds.
    #[error("invalid cursor position")]
    InvalidPosition,
}

/// A lightweight (row, column) cursor position.
///
/// `row` is 0-indexed line number, `col` is 0-indexed character offset within
/// the line. The cursor can rest one position past the last character of a line
/// (i.e. `col == line.len()`), which matches the behaviour of most editors.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Cursor {
    /// 0-indexed line number.
    pub row: usize,
    /// 0-indexed character offset within the line.
    pub col: usize,
}

impl fmt::Display for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.row, self.col)
    }
}

/// An editable text buffer with an embedded cursor.
///
/// Internally the buffer stores text as a vector of [`String`] lines (without
/// trailing newlines). An empty buffer contains a single empty line so that the
/// cursor always has a valid row to point at.
pub struct TextBuffer {
    /// The lines of text, each without a trailing newline.
    lines: Vec<String>,
    /// Current cursor position.
    cursor: Cursor,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBuffer {
    // ------------------------------------------------------------------
    // Construction & serialization
    // ------------------------------------------------------------------

    /// Creates a new empty buffer.
    ///
    /// The buffer starts with a single empty line and the cursor at `(0, 0)`.
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: Cursor::default(),
        }
    }

    /// Builds a buffer from the given text, splitting on `\n`.
    ///
    /// Trailing `\n` in the input does **not** produce an extra empty line,
    /// matching the convention of [`str::lines`].
    pub fn from_text(text: &str) -> Self {
        let lines: Vec<String> = if text.is_empty() {
            vec![String::new()]
        } else {
            text.lines().map(String::from).collect()
        };
        Self {
            lines,
            cursor: Cursor::default(),
        }
    }

    /// Flattens the buffer back into a single [`String`], joining lines with `\n`.
    pub fn to_text(&self) -> String {
        self.lines.join("\n")
    }

    // ------------------------------------------------------------------
    // Inspection
    // ------------------------------------------------------------------

    /// Returns the number of lines in the buffer (always >= 1).
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns a reference to the line at `index`, or `None` if out of bounds.
    pub fn get_line(&self, index: usize) -> Option<&str> {
        self.lines.get(index).map(String::as_str)
    }

    /// Returns an owned copy of the line at `row`, or an empty string if the
    /// row is out of bounds.
    pub fn line(&self, row: usize) -> String {
        match self.lines.get(row) {
            Some(s) => s.clone(),
            None => String::new(),
        }
    }

    /// Returns the current cursor position as `(row, col)`.
    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor.row, self.cursor.col)
    }

    // ------------------------------------------------------------------
    // Insertion
    // ------------------------------------------------------------------

    /// Inserts a single character at the cursor, advancing the cursor by one
    /// column. Inserting `'\n'` splits the current line at the cursor.
    pub fn insert_char(&mut self, ch: char) {
        let (row, col) = (self.cursor.row, self.cursor.col);
        if ch == '\n' {
            let remainder = self.lines[row].split_off(col);
            self.lines.insert(row + 1, remainder);
            self.cursor.row = row + 1;
            self.cursor.col = 0;
        } else {
            self.lines[row].insert(col, ch);
            self.cursor.col = col + 1;
        }
    }

    /// Inserts multi-line text at the cursor position.
    ///
    /// Newlines within `text` create new lines. The cursor ends at the
    /// column just past the last character of the inserted text on the final
    /// line of the insert.
    pub fn insert_text(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        let (row, col) = (self.cursor.row, self.cursor.col);

        // Split the current line at the cursor.
        let prefix = self.lines[row].clone();
        let (before, after) = prefix.split_at(col);
        let before = before.to_string();
        let after = after.to_string();

        let parts: Vec<&str> = text.split('\n').collect();
        let n_parts = parts.len();

        if n_parts == 1 {
            // Single-line insert.
            self.lines[row] = format!("{before}{text}{after}");
            self.cursor.col = col + text.chars().count();
        } else {
            // First segment appends to the prefix of the current line.
            self.lines[row] = format!("{before}{}", parts[0]);

            // Middle segments become standalone new lines.
            for (i, mid) in parts[1..n_parts - 1].iter().enumerate() {
                self.lines.insert(row + 1 + i, (*mid).to_string());
            }

            // Last segment: combine with the remainder of the original line.
            let last = parts[n_parts - 1];
            let last_line = format!("{last}{after}");
            let insert_at = row + n_parts - 1;
            self.lines.insert(insert_at, last_line);

            // Cursor ends at the column just past `last`.
            self.cursor.row = insert_at;
            self.cursor.col = last.chars().count();
        }
    }

    // ------------------------------------------------------------------
    // Deletion
    // ------------------------------------------------------------------

    /// Deletes the character **before** the cursor (backspace).
    ///
    /// If the cursor is at the start of a line that is not the first line, the
    /// current line is joined to the previous one and the cursor moves to the
    /// join point. If the cursor is at `(0, 0)` this is a no-op.
    pub fn backspace(&mut self) {
        let (row, col) = (self.cursor.row, self.cursor.col);
        if col > 0 {
            self.lines[row].remove(col - 1);
            self.cursor.col = col - 1;
        } else if row > 0 {
            let merged_col = self.lines[row - 1].len();
            let current = self.lines.remove(row);
            self.lines[row - 1].push_str(&current);
            self.cursor.row = row - 1;
            self.cursor.col = merged_col;
        }
    }

    /// Deletes the character **at** the cursor (forward delete).
    ///
    /// If the cursor is at the end of a line that is not the last line, the
    /// next line is joined to the current one. If the cursor is at the very end
    /// of the buffer this is a no-op.
    pub fn delete(&mut self) {
        let (row, col) = (self.cursor.row, self.cursor.col);
        if col < self.lines[row].len() {
            self.lines[row].remove(col);
        } else if row < self.lines.len() - 1 {
            let next = self.lines.remove(row + 1);
            self.lines[row].push_str(&next);
        }
    }

    // ------------------------------------------------------------------
    // Cursor movement
    // ------------------------------------------------------------------

    /// Moves the cursor to `(row, col)`, clamping to valid bounds.
    ///
    /// `row` is clamped to `[0, line_count-1]` and `col` is clamped to
    /// `[0, line[row].len()]`.
    pub fn move_cursor(&mut self, row: usize, col: usize) {
        let max_row = self.lines.len().saturating_sub(1);
        let row = row.min(max_row);
        let max_col = self.lines[row].len();
        let col = col.min(max_col);
        self.cursor = Cursor { row, col };
    }

    /// Moves the cursor left by one position, wrapping to the end of the
    /// previous line when at column 0.
    pub fn move_left(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.row > 0 {
            self.cursor.row -= 1;
            self.cursor.col = self.lines[self.cursor.row].len();
        }
    }

    /// Moves the cursor right by one position, wrapping to the start of the
    /// next line when at the end of the current line.
    pub fn move_right(&mut self) {
        if self.cursor.col < self.lines[self.cursor.row].len() {
            self.cursor.col += 1;
        } else if self.cursor.row < self.lines.len() - 1 {
            self.cursor.row += 1;
            self.cursor.col = 0;
        }
    }

    /// Moves the cursor up by one line, preserving column if possible (clamped
    /// to the shorter line's length).
    pub fn move_up(&mut self) {
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
            let max_col = self.lines[self.cursor.row].len();
            self.cursor.col = self.cursor.col.min(max_col);
        }
    }

    /// Moves the cursor down by one line, preserving column if possible (clamped
    /// to the shorter line's length).
    pub fn move_down(&mut self) {
        if self.cursor.row < self.lines.len() - 1 {
            self.cursor.row += 1;
            let max_col = self.lines[self.cursor.row].len();
            self.cursor.col = self.cursor.col.min(max_col);
        }
    }

    // ------------------------------------------------------------------
    // Misc
    // ------------------------------------------------------------------

    /// Clears all text, returning to a single empty line with cursor at `(0, 0)`.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.lines.push(String::new());
        self.cursor = Cursor::default();
    }
}
