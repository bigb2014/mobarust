//! Terminal grid model: cells, attributes, cursor, and grid operations.
//!
//! The grid is a row-major flat `Vec<Cell>` with fixed dimensions. The cursor
//! tracks the current write position. All operations are bounds-checked and
//! never panic on out-of-range inputs -- they clamp instead.

use serde::{Deserialize, Serialize};

/// ANSI color palette index (0-15 for standard, 0-255 for extended).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color(pub u8);

impl Default for Color {
    fn default() -> Self {
        Color::DEFAULT
    }
}

impl Color {
    /// Default foreground (white-ish, index 7 in the 16-color palette).
    pub const DEFAULT: Color = Color(7);
    /// Black.
    pub const BLACK: Color = Color(0);
    /// Red.
    pub const RED: Color = Color(1);
    /// Green.
    pub const GREEN: Color = Color(2);
    /// Yellow.
    pub const YELLOW: Color = Color(3);
    /// Blue.
    pub const BLUE: Color = Color(4);
    /// Magenta.
    pub const MAGENTA: Color = Color(5);
    /// Cyan.
    pub const CYAN: Color = Color(6);
    /// White.
    pub const WHITE: Color = Color(7);
}

/// Text cell attributes (bold, italic, underline, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attributes {
    /// Bold / increased intensity.
    pub bold: bool,
    /// Italic / slanted.
    pub italic: bool,
    /// Underlined.
    pub underline: bool,
    /// Reverse video (swap fg/bg).
    pub reverse: bool,
    /// Foreground color index.
    pub fg: Color,
    /// Background color index.
    pub bg: Color,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            reverse: false,
            fg: Color::DEFAULT,
            bg: Color::BLACK,
        }
    }
}

impl Attributes {
    /// Create a new `Attributes` with default colors and no styling.
    pub fn new() -> Self {
        Self::default()
    }
}

/// A single cell in the terminal grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    /// The character stored in this cell (space for empty cells).
    pub ch: char,
    /// Cell attributes.
    pub attrs: Attributes,
}

impl Cell {
    /// Create a blank cell (space character, default attributes).
    pub fn blank() -> Self {
        Self {
            ch: ' ',
            attrs: Attributes::default(),
        }
    }

    /// Create a cell with a character and the given attributes.
    pub fn with(ch: char, attrs: Attributes) -> Self {
        Self { ch, attrs }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::blank()
    }
}

/// Cursor position within the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Cursor {
    /// Zero-based row.
    pub row: usize,
    /// Zero-based column.
    pub col: usize,
}

/// A fixed-size terminal grid of cells.
///
/// The grid stores cells in a flat `Vec` in row-major order. Rows and columns
/// are zero-indexed. The cursor tracks the current write position.
#[derive(Debug, Clone)]
pub struct Grid {
    /// Number of rows.
    rows: usize,
    /// Number of columns.
    cols: usize,
    /// Flat cell storage (row-major).
    cells: Vec<Cell>,
    /// Current cursor position.
    cursor: Cursor,
}

/// Mode for clearing cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearMode {
    /// Clear from the cursor to the end (of line or screen).
    FromCursor,
    /// Clear from the start to the cursor (inclusive).
    ToCursor,
    /// Clear everything.
    All,
}

impl Grid {
    /// Create a new grid with the given dimensions, filled with blank cells.
    pub fn new(rows: usize, cols: usize) -> Self {
        let cells = vec![Cell::blank(); rows * cols];
        Self {
            rows,
            cols,
            cells,
            cursor: Cursor::default(),
        }
    }

    /// Number of rows in the grid.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of columns in the grid.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    /// Move the cursor to an absolute position (0-based).
    ///
    /// Out-of-bounds values are clamped to the grid boundary.
    pub fn move_cursor(&mut self, row: usize, col: usize) {
        self.cursor.row = row.min(self.rows.saturating_sub(1));
        self.cursor.col = col.min(self.cols.saturating_sub(1));
    }

    /// Get a reference to the cell at `(row, col)`.
    ///
    /// Returns `None` if the position is out of bounds.
    pub fn cell(&self, row: usize, col: usize) -> Option<&Cell> {
        if row < self.rows && col < self.cols {
            self.cells.get(row * self.cols + col)
        } else {
            None
        }
    }

    /// Get a mutable reference to the cell at `(row, col)`.
    fn cell_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        if row < self.rows && col < self.cols {
            let idx = row * self.cols + col;
            self.cells.get_mut(idx)
        } else {
            None
        }
    }

    /// Write a character at the current cursor position and advance the cursor.
    ///
    /// If the cursor is at the end of a row, it wraps to the next row. If the
    /// cursor is on the last row, the grid scrolls up by one line.
    pub fn write_char(&mut self, ch: char, attrs: Attributes) {
        if self.cursor.col >= self.cols {
            // Wrap to next line.
            self.cursor.col = 0;
            self.line_feed();
        }
        if let Some(cell) = self.cell_mut(self.cursor.row, self.cursor.col) {
            *cell = Cell::with(ch, attrs);
        }
        self.cursor.col += 1;
    }

    /// Move the cursor down by one row. If past the last row, scroll up.
    pub fn line_feed(&mut self) {
        if self.cursor.row + 1 >= self.rows {
            self.scroll_up(1);
        } else {
            self.cursor.row += 1;
        }
    }

    /// Move the cursor to column 0 (carriage return).
    pub fn carriage_return(&mut self) {
        self.cursor.col = 0;
    }

    /// Move the cursor left by one column (backspace), clamped at 0.
    pub fn backspace(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
    }

    /// Move the cursor up by `n` rows, clamped at row 0.
    pub fn cursor_up(&mut self, n: usize) {
        self.cursor.row = self.cursor.row.saturating_sub(n);
    }

    /// Move the cursor down by `n` rows, clamped at the last row.
    pub fn cursor_down(&mut self, n: usize) {
        self.cursor.row = (self.cursor.row + n).min(self.rows.saturating_sub(1));
    }

    /// Move the cursor forward (right) by `n` columns, clamped at the last col.
    pub fn cursor_forward(&mut self, n: usize) {
        self.cursor.col = (self.cursor.col + n).min(self.cols.saturating_sub(1));
    }

    /// Move the cursor back (left) by `n` columns, clamped at col 0.
    pub fn cursor_back(&mut self, n: usize) {
        self.cursor.col = self.cursor.col.saturating_sub(n);
    }

    /// Scroll the grid up by `n` lines.
    ///
    /// Top `n` lines are discarded, blank lines are appended at the bottom.
    pub fn scroll_up(&mut self, n: usize) {
        if n >= self.rows {
            self.cells.fill(Cell::blank());
            self.cursor.row = 0;
            return;
        }
        // Shift cells up by `n` rows.
        let shift = n * self.cols;
        self.cells.drain(0..shift);
        self.cells.resize(self.rows * self.cols, Cell::blank());
        // Keep cursor on the last row.
        self.cursor.row = self.rows.saturating_sub(1);
    }

    /// Clear part of the current line.
    pub fn clear_line(&mut self, mode: ClearMode) {
        let row = self.cursor.row;
        match mode {
            ClearMode::FromCursor => {
                for col in self.cursor.col..self.cols {
                    if let Some(cell) = self.cell_mut(row, col) {
                        *cell = Cell::blank();
                    }
                }
            }
            ClearMode::ToCursor => {
                for col in 0..=self.cursor.col.min(self.cols.saturating_sub(1)) {
                    if let Some(cell) = self.cell_mut(row, col) {
                        *cell = Cell::blank();
                    }
                }
            }
            ClearMode::All => {
                for col in 0..self.cols {
                    if let Some(cell) = self.cell_mut(row, col) {
                        *cell = Cell::blank();
                    }
                }
            }
        }
    }

    /// Clear part of the entire screen.
    pub fn clear_screen(&mut self, mode: ClearMode) {
        match mode {
            ClearMode::FromCursor => {
                // Clear from cursor to end of current row.
                let row = self.cursor.row;
                for col in self.cursor.col..self.cols {
                    if let Some(cell) = self.cell_mut(row, col) {
                        *cell = Cell::blank();
                    }
                }
                // Clear all rows below.
                for r in (self.cursor.row + 1)..self.rows {
                    for col in 0..self.cols {
                        if let Some(cell) = self.cell_mut(r, col) {
                            *cell = Cell::blank();
                        }
                    }
                }
            }
            ClearMode::ToCursor => {
                // Clear from start to cursor on current row.
                let row = self.cursor.row;
                for col in 0..=self.cursor.col.min(self.cols.saturating_sub(1)) {
                    if let Some(cell) = self.cell_mut(row, col) {
                        *cell = Cell::blank();
                    }
                }
                // Clear all rows above.
                for r in 0..self.cursor.row {
                    for col in 0..self.cols {
                        if let Some(cell) = self.cell_mut(r, col) {
                            *cell = Cell::blank();
                        }
                    }
                }
            }
            ClearMode::All => {
                self.cells.fill(Cell::blank());
            }
        }
    }

    /// Insert `n` blank lines at the cursor row, pushing existing lines down.
    pub fn insert_lines(&mut self, n: usize) {
        let row = self.cursor.row;
        if row >= self.rows {
            return;
        }
        let n = n.min(self.rows - row);
        // Build the new row content: blank lines at `row`, then existing lines.
        let mut new_cells = vec![Cell::blank(); self.rows * self.cols];
        // Copy rows above `row` unchanged.
        for r in 0..row {
            for c in 0..self.cols {
                let src = r * self.cols + c;
                let dst = r * self.cols + c;
                new_cells[dst] = self.cells[src];
            }
        }
        // Copy rows from `row` to `rows - n` down by `n`.
        for r in 0..(self.rows - row - n) {
            for c in 0..self.cols {
                let src = (row + r) * self.cols + c;
                let dst = (row + n + r) * self.cols + c;
                new_cells[dst] = self.cells[src];
            }
        }
        // Rows `row..row+n` are already blank.
        self.cells = new_cells;
        self.cursor.col = 0;
    }

    /// Delete `n` lines at the cursor row, pulling lines below up.
    pub fn delete_lines(&mut self, n: usize) {
        let row = self.cursor.row;
        if row >= self.rows {
            return;
        }
        let n = n.min(self.rows - row);
        let mut new_cells = vec![Cell::blank(); self.rows * self.cols];
        // Copy rows above `row` unchanged.
        for r in 0..row {
            for c in 0..self.cols {
                new_cells[r * self.cols + c] = self.cells[r * self.cols + c];
            }
        }
        // Copy rows from `row + n` to end, shifted up by `n`.
        for r in 0..(self.rows - row - n) {
            for c in 0..self.cols {
                let src = (row + n + r) * self.cols + c;
                let dst = (row + r) * self.cols + c;
                new_cells[dst] = self.cells[src];
            }
        }
        // Bottom `n` rows are already blank.
        self.cells = new_cells;
        self.cursor.col = 0;
    }

    /// Insert `n` blank characters at the cursor, shifting remaining chars right.
    pub fn insert_chars(&mut self, n: usize) {
        let row = self.cursor.row;
        let col = self.cursor.col;
        if row >= self.rows || col >= self.cols {
            return;
        }
        let n = n.min(self.cols - col);
        // Work on a copy of the row to avoid borrow conflicts.
        let mut row_cells: Vec<Cell> = (0..self.cols)
            .map(|c| self.cell(row, c).copied().unwrap_or_default())
            .collect();
        // Shift cells right from the end.
        for c in (col + n..self.cols).rev() {
            row_cells[c] = row_cells[c - n];
        }
        // Blank the inserted region.
        for cell in row_cells
            .iter_mut()
            .skip(col)
            .take((col + n).min(self.cols) - col)
        {
            *cell = Cell::blank();
        }
        // Write back.
        for (c, &new_cell) in row_cells.iter().enumerate().take(self.cols) {
            if let Some(cell) = self.cell_mut(row, c) {
                *cell = new_cell;
            }
        }
    }

    /// Delete `n` characters at the cursor, shifting remaining chars left.
    pub fn delete_chars(&mut self, n: usize) {
        let row = self.cursor.row;
        let col = self.cursor.col;
        if row >= self.rows || col >= self.cols {
            return;
        }
        let n = n.min(self.cols - col);
        // Work on a copy of the row to avoid borrow conflicts.
        let mut row_cells: Vec<Cell> = (0..self.cols)
            .map(|c| self.cell(row, c).copied().unwrap_or_default())
            .collect();
        // Shift cells left.
        for c in col..(self.cols - n) {
            row_cells[c] = row_cells[c + n];
        }
        // Blank the vacated cells at the end.
        for cell in row_cells.iter_mut().skip(self.cols - n) {
            *cell = Cell::blank();
        }
        // Write back.
        for (c, &new_cell) in row_cells.iter().enumerate().take(self.cols) {
            if let Some(cell) = self.cell_mut(row, c) {
                *cell = new_cell;
            }
        }
    }

    /// Render the grid as a string, row by row, with trailing spaces trimmed.
    ///
    /// Useful for testing and debugging.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for r in 0..self.rows {
            let mut row_str = String::new();
            for c in 0..self.cols {
                if let Some(cell) = self.cell(r, c) {
                    row_str.push(cell.ch);
                }
            }
            // Trim trailing blanks for readability.
            let trimmed = row_str.trim_end();
            out.push_str(trimmed);
            if r + 1 < self.rows {
                out.push('\n');
            }
        }
        out
    }

    /// Resize the grid to `new_rows` x `new_cols`.
    ///
    /// Existing content that fits within the new dimensions is preserved.
    /// Cells in the expanded region are filled with blank cells. Content
    /// that no longer fits (rows or columns beyond the new bounds) is
    /// discarded -- lines are truncated, not rewrapped. The cursor is
    /// clamped to the new bounds.
    pub fn resize(&mut self, new_rows: usize, new_cols: usize) {
        // Build the new cell buffer by copying overlapping cells.
        let mut new_cells = vec![Cell::blank(); new_rows * new_cols];
        let copy_rows = self.rows.min(new_rows);
        let copy_cols = self.cols.min(new_cols);
        for r in 0..copy_rows {
            for c in 0..copy_cols {
                let src = r * self.cols + c;
                let dst = r * new_cols + c;
                new_cells[dst] = self.cells[src];
            }
        }
        self.rows = new_rows;
        self.cols = new_cols;
        self.cells = new_cells;
        // Clamp cursor to new bounds.
        self.cursor.row = self.cursor.row.min(self.rows.saturating_sub(1));
        self.cursor.col = self.cursor.col.min(self.cols.saturating_sub(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Grid creation and basic cell access ---

    #[test]
    fn grid_create_and_read_cell() {
        let mut g = Grid::new(3, 5);
        // Initially all blank.
        assert_eq!(g.cell(0, 0).map(|c| c.ch), Some(' '));
        // Write a char at (0,0).
        g.move_cursor(0, 0);
        g.write_char('H', Attributes::default());
        assert_eq!(g.cell(0, 0).map(|c| c.ch), Some('H'));
    }

    #[test]
    fn cursor_moves_right_on_write() {
        let mut g = Grid::new(2, 5);
        g.write_char('A', Attributes::default());
        assert_eq!(g.cursor(), Cursor { row: 0, col: 1 });
        g.write_char('B', Attributes::default());
        assert_eq!(g.cursor(), Cursor { row: 0, col: 2 });
        assert_eq!(g.cell(0, 0).map(|c| c.ch), Some('A'));
        assert_eq!(g.cell(0, 1).map(|c| c.ch), Some('B'));
    }

    #[test]
    fn cursor_wraps_at_end_of_row() {
        let mut g = Grid::new(3, 3);
        g.move_cursor(0, 2);
        // Cursor at last col; writing places char and advances col past end.
        g.write_char('X', Attributes::default());
        assert_eq!(g.cell(0, 2).map(|c| c.ch), Some('X'));
        // Col is now 3 (past end); next write wraps to next row.
        assert_eq!(g.cursor(), Cursor { row: 0, col: 3 });
        g.write_char('Y', Attributes::default());
        // After wrap, cursor is at next row, col 1 (col advanced after write).
        assert_eq!(g.cursor(), Cursor { row: 1, col: 1 });
        assert_eq!(g.cell(1, 0).map(|c| c.ch), Some('Y'));
    }

    #[test]
    fn cursor_move_to_arbitrary_position() {
        let mut g = Grid::new(5, 10);
        g.move_cursor(3, 7);
        assert_eq!(g.cursor(), Cursor { row: 3, col: 7 });
        // Out of bounds clamps.
        g.move_cursor(100, 100);
        assert_eq!(g.cursor(), Cursor { row: 4, col: 9 });
    }

    // --- Clear operations ---

    #[test]
    fn clear_line_from_cursor() {
        let mut g = Grid::new(2, 5);
        for c in 0..5 {
            g.move_cursor(0, c);
            g.write_char((b'A' + c as u8) as char, Attributes::default());
        }
        g.move_cursor(0, 2);
        g.clear_line(ClearMode::FromCursor);
        assert_eq!(g.cell(0, 0).map(|c| c.ch), Some('A'));
        assert_eq!(g.cell(0, 1).map(|c| c.ch), Some('B'));
        assert_eq!(g.cell(0, 2).map(|c| c.ch), Some(' '));
        assert_eq!(g.cell(0, 4).map(|c| c.ch), Some(' '));
    }

    #[test]
    fn clear_line_all() {
        let mut g = Grid::new(2, 5);
        for c in 0..5 {
            g.move_cursor(0, c);
            g.write_char((b'A' + c as u8) as char, Attributes::default());
        }
        g.move_cursor(0, 0);
        g.clear_line(ClearMode::All);
        for c in 0..5 {
            assert_eq!(g.cell(0, c).map(|x| x.ch), Some(' '));
        }
    }

    #[test]
    fn clear_screen_all() {
        let mut g = Grid::new(3, 3);
        g.move_cursor(0, 0);
        g.write_char('X', Attributes::default());
        g.move_cursor(1, 1);
        g.write_char('Y', Attributes::default());
        g.clear_screen(ClearMode::All);
        for r in 0..3 {
            for c in 0..3 {
                assert_eq!(g.cell(r, c).map(|x| x.ch), Some(' '));
            }
        }
    }

    #[test]
    fn clear_screen_from_cursor() {
        let mut g = Grid::new(3, 3);
        // Fill row 0.
        g.move_cursor(0, 0);
        g.write_char('A', Attributes::default());
        g.write_char('B', Attributes::default());
        g.write_char('C', Attributes::default());
        // Fill row 1.
        g.move_cursor(1, 0);
        g.write_char('D', Attributes::default());
        g.write_char('E', Attributes::default());
        g.write_char('F', Attributes::default());
        // Clear from (0,1) to end of screen.
        g.move_cursor(0, 1);
        g.clear_screen(ClearMode::FromCursor);
        assert_eq!(g.cell(0, 0).map(|c| c.ch), Some('A'));
        assert_eq!(g.cell(0, 1).map(|c| c.ch), Some(' '));
        assert_eq!(g.cell(0, 2).map(|c| c.ch), Some(' '));
        assert_eq!(g.cell(1, 0).map(|c| c.ch), Some(' '));
        assert_eq!(g.cell(1, 2).map(|c| c.ch), Some(' '));
    }

    // --- Scroll ---

    #[test]
    fn scroll_up_when_past_last_row() {
        let mut g = Grid::new(3, 3);
        // Fill row 0 with 'A', row 1 with 'B'.
        g.move_cursor(0, 0);
        g.write_char('A', Attributes::default());
        g.write_char('A', Attributes::default());
        g.write_char('A', Attributes::default());
        // Cursor now at (0,3) - past end. Next write wraps.
        g.write_char('B', Attributes::default()); // wraps to (1,0), writes B
        g.write_char('B', Attributes::default());
        g.write_char('B', Attributes::default());
        // Cursor at (1,3). Next write wraps to (2,0).
        g.write_char('C', Attributes::default()); // wraps to (2,0)
                                                  // Now at (2,1). Line feed from last row should scroll.
        g.line_feed();
        // Row 0 (was 'A') should be gone, row 0 should now be 'B'.
        assert_eq!(g.cell(0, 0).map(|c| c.ch), Some('B'));
        assert_eq!(g.cell(1, 0).map(|c| c.ch), Some('C'));
    }

    #[test]
    fn scroll_up_explicit() {
        let mut g = Grid::new(3, 2);
        g.move_cursor(0, 0);
        g.write_char('A', Attributes::default());
        g.write_char('A', Attributes::default());
        g.move_cursor(1, 0);
        g.write_char('B', Attributes::default());
        g.write_char('B', Attributes::default());
        g.move_cursor(2, 0);
        g.write_char('C', Attributes::default());
        g.write_char('C', Attributes::default());
        g.scroll_up(1);
        assert_eq!(g.cell(0, 0).map(|c| c.ch), Some('B'));
        assert_eq!(g.cell(1, 0).map(|c| c.ch), Some('C'));
        assert_eq!(g.cell(2, 0).map(|c| c.ch), Some(' '));
    }

    // --- Resize / reflow ---

    #[test]
    fn resize_larger_adds_blank_rows() {
        let mut g = Grid::new(2, 5);
        // Fill row 0 with "ABCDE".
        for c in 0..5 {
            g.move_cursor(0, c);
            g.write_char((b'A' + c as u8) as char, Attributes::default());
        }
        // Fill row 1 with "FGHIJ".
        for c in 0..5 {
            g.move_cursor(1, c);
            g.write_char((b'F' + c as u8) as char, Attributes::default());
        }
        g.resize(4, 5);
        assert_eq!(g.rows(), 4);
        assert_eq!(g.cols(), 5);
        // Old content preserved.
        assert_eq!(g.cell(0, 0).map(|c| c.ch), Some('A'));
        assert_eq!(g.cell(0, 4).map(|c| c.ch), Some('E'));
        assert_eq!(g.cell(1, 0).map(|c| c.ch), Some('F'));
        assert_eq!(g.cell(1, 4).map(|c| c.ch), Some('J'));
        // New rows are blank.
        for c in 0..5 {
            assert_eq!(g.cell(2, c).map(|c| c.ch), Some(' '));
            assert_eq!(g.cell(3, c).map(|c| c.ch), Some(' '));
        }
    }

    #[test]
    fn resize_smaller_truncates() {
        let mut g = Grid::new(4, 5);
        // Fill all 4 rows with distinct characters.
        for r in 0..4 {
            for c in 0..5 {
                g.move_cursor(r, c);
                g.write_char((b'A' + (r * 5 + c) as u8) as char, Attributes::default());
            }
        }
        g.resize(2, 5);
        assert_eq!(g.rows(), 2);
        assert_eq!(g.cols(), 5);
        // Only first 2 rows remain.
        assert_eq!(g.cell(0, 0).map(|c| c.ch), Some('A'));
        assert_eq!(g.cell(0, 4).map(|c| c.ch), Some('E'));
        assert_eq!(g.cell(1, 0).map(|c| c.ch), Some('F'));
        assert_eq!(g.cell(1, 4).map(|c| c.ch), Some('J'));
        // Rows 2 and 3 are gone.
        assert!(g.cell(2, 0).is_none());
    }

    #[test]
    fn resize_wider_adds_blank_cols() {
        let mut g = Grid::new(2, 3);
        // Fill row 0 with "ABC".
        for c in 0..3 {
            g.move_cursor(0, c);
            g.write_char((b'A' + c as u8) as char, Attributes::default());
        }
        // Fill row 1 with "DEF".
        for c in 0..3 {
            g.move_cursor(1, c);
            g.write_char((b'D' + c as u8) as char, Attributes::default());
        }
        g.resize(2, 5);
        assert_eq!(g.rows(), 2);
        assert_eq!(g.cols(), 5);
        // Old content preserved.
        assert_eq!(g.cell(0, 0).map(|c| c.ch), Some('A'));
        assert_eq!(g.cell(0, 2).map(|c| c.ch), Some('C'));
        assert_eq!(g.cell(1, 0).map(|c| c.ch), Some('D'));
        assert_eq!(g.cell(1, 2).map(|c| c.ch), Some('F'));
        // New columns are blank.
        assert_eq!(g.cell(0, 3).map(|c| c.ch), Some(' '));
        assert_eq!(g.cell(0, 4).map(|c| c.ch), Some(' '));
        assert_eq!(g.cell(1, 3).map(|c| c.ch), Some(' '));
        assert_eq!(g.cell(1, 4).map(|c| c.ch), Some(' '));
    }

    #[test]
    fn resize_narrower_truncates_cols() {
        let mut g = Grid::new(2, 5);
        // Fill row 0 with "ABCDE".
        for c in 0..5 {
            g.move_cursor(0, c);
            g.write_char((b'A' + c as u8) as char, Attributes::default());
        }
        // Fill row 1 with "FGHIJ".
        for c in 0..5 {
            g.move_cursor(1, c);
            g.write_char((b'F' + c as u8) as char, Attributes::default());
        }
        g.resize(2, 3);
        assert_eq!(g.rows(), 2);
        assert_eq!(g.cols(), 3);
        // Only first 3 columns remain.
        assert_eq!(g.cell(0, 0).map(|c| c.ch), Some('A'));
        assert_eq!(g.cell(0, 2).map(|c| c.ch), Some('C'));
        assert_eq!(g.cell(1, 0).map(|c| c.ch), Some('F'));
        assert_eq!(g.cell(1, 2).map(|c| c.ch), Some('H'));
        // Column 3 and beyond are gone.
        assert!(g.cell(0, 3).is_none());
    }

    #[test]
    fn resize_clamps_cursor() {
        let mut g = Grid::new(5, 5);
        g.move_cursor(4, 4);
        assert_eq!(g.cursor(), Cursor { row: 4, col: 4 });
        g.resize(3, 3);
        assert_eq!(g.rows(), 3);
        assert_eq!(g.cols(), 3);
        // Cursor clamped to new bounds: (2, 2).
        assert_eq!(g.cursor(), Cursor { row: 2, col: 2 });
    }

    #[test]
    fn resize_preserves_content() {
        let mut g = Grid::new(3, 3);
        // Fill with a pattern: each cell = 'A' + (row*3 + col).
        for r in 0..3 {
            for c in 0..3 {
                g.move_cursor(r, c);
                g.write_char((b'A' + (r * 3 + c) as u8) as char, Attributes::default());
            }
        }
        // Resize larger: 5x5.
        g.resize(5, 5);
        assert_eq!(g.rows(), 5);
        assert_eq!(g.cols(), 5);
        // Original 3x3 content survives.
        for r in 0..3 {
            for c in 0..3 {
                let expected = (b'A' + (r * 3 + c) as u8) as char;
                assert_eq!(g.cell(r, c).map(|cell| cell.ch), Some(expected));
            }
        }
        // New cells are blank.
        assert_eq!(g.cell(3, 0).map(|c| c.ch), Some(' '));
        assert_eq!(g.cell(4, 4).map(|c| c.ch), Some(' '));
        assert_eq!(g.cell(0, 3).map(|c| c.ch), Some(' '));
        // Resize back smaller: 3x3.
        g.resize(3, 3);
        assert_eq!(g.rows(), 3);
        assert_eq!(g.cols(), 3);
        // Original 3x3 content still survives.
        for r in 0..3 {
            for c in 0..3 {
                let expected = (b'A' + (r * 3 + c) as u8) as char;
                assert_eq!(g.cell(r, c).map(|cell| cell.ch), Some(expected));
            }
        }
    }
}
