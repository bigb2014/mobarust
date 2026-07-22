//! Text selection model for the terminal grid.
//!
//! Supports both linear (stream) and rectangular (block) selection modes.
//! Selection start and end positions can be in any order; they are
//! normalized internally so that `start <= end`.

use serde::{Deserialize, Serialize};

/// A position in the terminal grid: `(row, col)`.
pub type Position = (usize, usize);

/// The mode of a text selection.
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum SelectionMode {
    /// Linear/stream selection: text flows from start to end.
    Linear,
    /// Rectangular/block selection: a column-aligned block of text.
    Block,
}

/// A trait for types that can provide row text to the selection extractor.
///
/// This abstracts over the grid so that the selection model can be tested
/// independently of the grid implementation.
pub trait TextGrid {
    /// Returns the text of the given row, or `None` if the row is out of bounds.
    fn row_text(&self, row: usize) -> Option<&str>;

    /// Returns the length (in bytes/chars) of the given row, or 0 if out of bounds.
    fn row_len(&self, row: usize) -> usize;

    /// Returns the number of rows in the grid.
    fn num_rows(&self) -> usize;
}

/// A text selection with start and end positions.
///
/// The selection can be in linear (stream) or rectangular (block) mode.
/// Start and end may be in any order; use [`normalized`](Self::normalized)
/// to get them in canonical `(start <= end)` order.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Selection {
    /// The raw start position (may be after end).
    start: Position,
    /// The raw end position (may be before start).
    end: Position,
    /// The selection mode.
    mode: SelectionMode,
}

impl Selection {
    /// Creates a new linear selection from `start` to `end`.
    #[must_use]
    pub fn new(start: Position, end: Position) -> Self {
        Self {
            start,
            end,
            mode: SelectionMode::Linear,
        }
    }

    /// Creates a new rectangular (block) selection from `start` to `end`.
    #[must_use]
    pub fn new_block(start: Position, end: Position) -> Self {
        Self {
            start,
            end,
            mode: SelectionMode::Block,
        }
    }

    /// Returns the selection mode.
    #[must_use]
    pub fn mode(&self) -> SelectionMode {
        self.mode
    }

    /// Returns `true` if the selection is empty (start == end).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns the normalized (start, end) pair where `start <= end`.
    ///
    /// For linear mode, normalization compares positions lexicographically
    /// (row first, then column). For block mode, the start corner is
    /// `(min_row, min_col)` and the end corner is `(max_row, max_col)`.
    #[must_use]
    pub fn normalized(&self) -> (Position, Position) {
        match self.mode {
            SelectionMode::Linear => {
                if self.start <= self.end {
                    (self.start, self.end)
                } else {
                    (self.end, self.start)
                }
            }
            SelectionMode::Block => {
                let min_row = self.start.0.min(self.end.0);
                let max_row = self.start.0.max(self.end.0);
                let min_col = self.start.1.min(self.end.1);
                let max_col = self.start.1.max(self.end.1);
                ((min_row, min_col), (max_row, max_col))
            }
        }
    }

    /// Extracts the selected text from a grid-like structure.
    ///
    /// For linear selections, the text spans from the start position to
    /// the end position, with newlines between rows. The end column is
    /// exclusive (like Rust slice ranges).
    ///
    /// For block selections, each row within the row range contributes
    /// the substring from `min_col` to `max_col` (exclusive), joined by
    /// newlines.
    ///
    /// Returns an empty string if the selection is empty or the grid
    /// has no relevant content.
    pub fn extract_text(&self, grid: &dyn TextGrid) -> String {
        if self.is_empty() {
            return String::new();
        }

        match self.mode {
            SelectionMode::Linear => self.extract_linear(grid),
            SelectionMode::Block => self.extract_block(grid),
        }
    }

    /// Extracts text for a linear (stream) selection.
    fn extract_linear(&self, grid: &dyn TextGrid) -> String {
        let (start, end) = self.normalized();
        let (start_row, start_col) = start;
        let (end_row, end_col) = end;

        // Clamp end_row to the grid's row count.
        let max_row = grid.num_rows().saturating_sub(1);
        let end_row = end_row.min(max_row);

        let mut result = String::new();

        for row in start_row..=end_row {
            let row_text = grid.row_text(row).unwrap_or("");
            let row_len = grid.row_len(row);

            if row == start_row && row == end_row {
                // Single row: slice from start_col to end_col.
                let s = start_col.min(row_len);
                let e = end_col.min(row_len);
                result.push_str(&row_text[s..e]);
            } else if row == start_row {
                // First row: from start_col to end of row.
                let s = start_col.min(row_len);
                result.push_str(&row_text[s..]);
                result.push('\n');
            } else if row == end_row {
                // Last row: from beginning to end_col.
                let e = end_col.min(row_len);
                result.push_str(&row_text[..e]);
            } else {
                // Middle row: entire row.
                result.push_str(row_text);
                result.push('\n');
            }
        }

        result
    }

    /// Extracts text for a rectangular (block) selection.
    fn extract_block(&self, grid: &dyn TextGrid) -> String {
        let (start, end) = self.normalized();
        let (start_row, start_col) = start;
        let (end_row, end_col) = end;

        // Clamp end_row to the grid's row count.
        let max_row = grid.num_rows().saturating_sub(1);
        let end_row = end_row.min(max_row);

        let mut result = String::new();

        for row in start_row..=end_row {
            let row_text = grid.row_text(row).unwrap_or("");
            let row_len = grid.row_len(row);

            let s = start_col.min(row_len);
            let e = end_col.min(row_len);
            result.push_str(&row_text[s..e]);

            if row < end_row {
                result.push('\n');
            }
        }

        result
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new((0, 0), (0, 0))
    }
}
