//! VT parser: processes VT100/ANSI escape sequences into grid operations.
//!
//! Uses the `vte` crate for low-level byte parsing and implements
//! `vte::Perform` to apply actions to the terminal grid.

use crate::grid::{Attributes, ClearMode, Grid};
use vte::{Params, Perform};

/// SGR color codes (ANSI 3-bit/4-bit).
const SGR_FG_OFFSET: u16 = 30;
const SGR_FG_BRIGHT_OFFSET: u16 = 90;
const SGR_BG_OFFSET: u16 = 40;
const SGR_BG_BRIGHT_OFFSET: u16 = 100;

/// The terminal: combines a grid with a VT parser.
///
/// Implements `vte::Perform` to process parsed escape sequences
/// and apply them to the grid.
pub struct Terminal {
    /// The visible grid.
    pub grid: Grid,
    /// Current text attributes (applied to new writes).
    attrs: Attributes,
}

impl Terminal {
    /// Creates a new terminal with the given dimensions.
    #[must_use]
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            grid: Grid::new(rows, cols),
            attrs: Attributes::default(),
        }
    }

    /// Processes a byte slice through the VT parser and applies to the grid.
    pub fn process(&mut self, bytes: &[u8]) {
        let mut parser = vte::Parser::new();
        parser.advance(self, bytes);
    }

    /// Processes a string through the VT parser.
    pub fn process_str(&mut self, s: &str) {
        self.process(s.as_bytes());
    }

    /// Returns the character at a grid position, or space if out of bounds.
    #[must_use]
    pub fn char_at(&self, row: usize, col: usize) -> char {
        self.grid.cell(row, col).map_or(' ', |c| c.ch)
    }

    /// Applies an SGR (Select Graphic Rendition) parameter list to current attrs.
    fn apply_sgr(&mut self, params: &Params) {
        for param in params {
            if param.is_empty() {
                self.attrs = Attributes::default();
                continue;
            }
            for &code in param {
                match code {
                    0 => self.attrs = Attributes::default(),
                    1 => self.attrs.bold = true,
                    3 => self.attrs.italic = true,
                    4 => self.attrs.underline = true,
                    7 => self.attrs.reverse = true,
                    22 => self.attrs.bold = false,
                    23 => self.attrs.italic = false,
                    24 => self.attrs.underline = false,
                    27 => self.attrs.reverse = false,
                    30..=37 => {
                        self.attrs.fg = crate::grid::Color((code - SGR_FG_OFFSET) as u8);
                    }
                    38 => {} // extended fg - skip for now
                    39 => self.attrs.fg = crate::grid::Color::DEFAULT,
                    40..=47 => {
                        self.attrs.bg = crate::grid::Color((code - SGR_BG_OFFSET) as u8);
                    }
                    48 => {} // extended bg - skip for now
                    49 => self.attrs.bg = crate::grid::Color::default(),
                    90..=97 => {
                        self.attrs.fg = crate::grid::Color((code - SGR_FG_BRIGHT_OFFSET + 8) as u8);
                    }
                    100..=107 => {
                        self.attrs.bg = crate::grid::Color((code - SGR_BG_BRIGHT_OFFSET + 8) as u8);
                    }
                    _ => {}
                }
            }
        }
    }

    /// Extracts a single parameter value, defaulting to 1 if empty/missing.
    fn param_or_default(params: &Params, index: usize, default: u16) -> u16 {
        for (i, param) in params.into_iter().enumerate() {
            if i == index {
                if param.is_empty() {
                    return default;
                }
                return param[0];
            }
        }
        default
    }
}

impl Perform for Terminal {
    fn print(&mut self, ch: char) {
        self.grid.write_char(ch, self.attrs);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            // Line feed
            b'\n' => self.grid.line_feed(),
            // Carriage return
            b'\r' => self.grid.carriage_return(),
            // Backspace
            0x08 => self.grid.backspace(),
            // Tab - move to next 8-column tab stop
            0x09 => {
                let next_tab = (self.grid.cursor().col + 8) & !7;
                self.grid.move_cursor(self.grid.cursor().row, next_tab);
            }
            // Bell - ignore for now
            0x07 => {}
            // Vertical tab / form feed - treat as line feed
            0x0B | 0x0C => self.grid.line_feed(),
            _ => {}
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, byte: char) {
        match byte {
            // Cursor position: ESC [ row ; col H
            'H' | 'f' => {
                let row = Self::param_or_default(params, 0, 1);
                let col = Self::param_or_default(params, 1, 1);
                // CSI params are 1-based; convert to 0-based
                self.grid.move_cursor(
                    (row as usize).saturating_sub(1),
                    (col as usize).saturating_sub(1),
                );
            }
            // Cursor up
            'A' => {
                let n = Self::param_or_default(params, 0, 1);
                self.grid.cursor_up(n as usize);
            }
            // Cursor down
            'B' => {
                let n = Self::param_or_default(params, 0, 1);
                self.grid.cursor_down(n as usize);
            }
            // Cursor forward
            'C' => {
                let n = Self::param_or_default(params, 0, 1);
                self.grid.cursor_forward(n as usize);
            }
            // Cursor back
            'D' => {
                let n = Self::param_or_default(params, 0, 1);
                self.grid.cursor_back(n as usize);
            }
            // Erase display
            'J' => {
                let mode = Self::param_or_default(params, 0, 0);
                match mode {
                    0 => self.grid.clear_screen(ClearMode::FromCursor),
                    1 => self.grid.clear_screen(ClearMode::ToCursor),
                    2 => self.grid.clear_screen(ClearMode::All),
                    _ => {}
                }
            }
            // Erase line
            'K' => {
                let mode = Self::param_or_default(params, 0, 0);
                match mode {
                    0 => self.grid.clear_line(ClearMode::FromCursor),
                    1 => self.grid.clear_line(ClearMode::ToCursor),
                    2 => self.grid.clear_line(ClearMode::All),
                    _ => {}
                }
            }
            // Insert lines
            'L' => {
                let n = Self::param_or_default(params, 0, 1);
                self.grid.insert_lines(n as usize);
            }
            // Delete lines
            'M' => {
                let n = Self::param_or_default(params, 0, 1);
                self.grid.delete_lines(n as usize);
            }
            // Insert chars
            '@' => {
                let n = Self::param_or_default(params, 0, 1);
                self.grid.insert_chars(n as usize);
            }
            // Delete chars
            'P' => {
                let n = Self::param_or_default(params, 0, 1);
                self.grid.delete_chars(n as usize);
            }
            // SGR (Select Graphic Rendition)
            'm' => self.apply_sgr(params),
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // Handle ESC sequences (e.g., ESC c = reset terminal)
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // Handle OSC sequences (e.g., set window title)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_appears_in_grid() {
        let mut term = Terminal::new(10, 80);
        term.process_str("Hello");
        assert_eq!(term.char_at(0, 0), 'H');
        assert_eq!(term.char_at(0, 1), 'e');
        assert_eq!(term.char_at(0, 2), 'l');
        assert_eq!(term.char_at(0, 3), 'l');
        assert_eq!(term.char_at(0, 4), 'o');
    }

    #[test]
    fn newline_moves_cursor_down() {
        let mut term = Terminal::new(10, 80);
        term.process_str("Hi\r\nWorld");
        assert_eq!(term.char_at(0, 0), 'H');
        assert_eq!(term.char_at(0, 1), 'i');
        assert_eq!(term.char_at(1, 0), 'W');
        assert_eq!(term.char_at(1, 1), 'o');
    }

    #[test]
    fn carriage_return_moves_to_col_zero() {
        let mut term = Terminal::new(10, 80);
        term.process_str("Hi\rX");
        assert_eq!(term.char_at(0, 0), 'X');
        assert_eq!(term.char_at(0, 1), 'i');
    }

    #[test]
    fn backspace_moves_cursor_left() {
        let mut term = Terminal::new(10, 80);
        term.process_str("Hi\x08X");
        assert_eq!(term.char_at(0, 0), 'H');
        assert_eq!(term.char_at(0, 1), 'X');
    }

    #[test]
    fn csi_cursor_position() {
        let mut term = Terminal::new(10, 80);
        term.process_str("\x1b[5;10H");
        assert_eq!(term.grid.cursor().row, 4);
        assert_eq!(term.grid.cursor().col, 9);
    }

    #[test]
    fn csi_cursor_up() {
        let mut term = Terminal::new(10, 80);
        term.process_str("\x1b[5;10H\x1b[2A");
        assert_eq!(term.grid.cursor().row, 2);
        assert_eq!(term.grid.cursor().col, 9);
    }

    #[test]
    fn csi_cursor_down() {
        let mut term = Terminal::new(10, 80);
        term.process_str("\x1b[2B");
        assert_eq!(term.grid.cursor().row, 2);
    }

    #[test]
    fn csi_cursor_forward() {
        let mut term = Terminal::new(10, 80);
        term.process_str("\x1b[5C");
        assert_eq!(term.grid.cursor().col, 5);
    }

    #[test]
    fn csi_cursor_back() {
        let mut term = Terminal::new(10, 80);
        term.process_str("\x1b[10C\x1b[3D");
        assert_eq!(term.grid.cursor().col, 7);
    }

    #[test]
    fn sgr_sets_foreground_red() {
        let mut term = Terminal::new(10, 80);
        term.process_str("\x1b[31mR");
        let cell = term.grid.cell(0, 0).unwrap();
        assert_eq!(cell.attrs.fg, crate::grid::Color::RED);
    }

    #[test]
    fn sgr_reset_clears_style() {
        let mut term = Terminal::new(10, 80);
        term.process_str("\x1b[31m\x1b[0mR");
        let cell = term.grid.cell(0, 0).unwrap();
        assert_eq!(cell.attrs.fg, crate::grid::Color::DEFAULT);
    }

    #[test]
    fn sgr_bold_sets_attribute() {
        let mut term = Terminal::new(10, 80);
        term.process_str("\x1b[1mB");
        let cell = term.grid.cell(0, 0).unwrap();
        assert!(cell.attrs.bold);
    }

    #[test]
    fn sgr_bright_foreground() {
        let mut term = Terminal::new(10, 80);
        term.process_str("\x1b[91mB");
        let cell = term.grid.cell(0, 0).unwrap();
        assert_eq!(cell.attrs.fg.0, 9); // bright red = 91 - 90 + 8 = 9
    }

    #[test]
    fn erase_display_clears_screen() {
        let mut term = Terminal::new(5, 10);
        term.process_str("Hello\nWorld\nTest");
        term.process_str("\x1b[2J");
        // All cells should be blank
        for row in 0..5 {
            for col in 0..10 {
                assert!(term.grid.cell(row, col).unwrap().ch == ' ');
            }
        }
    }

    #[test]
    fn erase_line_clears_current_line() {
        let mut term = Terminal::new(5, 10);
        term.process_str("Hello");
        term.process_str("\x1b[2K");
        for col in 0..10 {
            assert!(term.grid.cell(0, col).unwrap().ch == ' ');
        }
    }

    #[test]
    fn erase_line_from_cursor() {
        let mut term = Terminal::new(5, 10);
        term.process_str("Hello");
        // Move cursor to col 2
        term.process_str("\x1b[1;3H");
        term.process_str("\x1b[0K");
        // First 2 chars should remain
        assert_eq!(term.char_at(0, 0), 'H');
        assert_eq!(term.char_at(0, 1), 'e');
        // Rest should be blank
        assert!(term.grid.cell(0, 2).unwrap().ch == ' ');
    }

    #[test]
    fn text_wraps_at_end_of_row() {
        let mut term = Terminal::new(5, 5);
        term.process_str("ABCDE");
        // 'A'-'E' fills row 0
        assert_eq!(term.char_at(0, 0), 'A');
        assert_eq!(term.char_at(0, 4), 'E');
    }

    #[test]
    fn scroll_on_overflow() {
        let mut term = Terminal::new(3, 5);
        term.process_str("AAA\r\nBBB\r\nCCC\r\nDDD");
        // After 3 newlines, the grid should have scrolled
        // Row 0 should contain BBB, row 1 CCC, row 2 DDD
        assert_eq!(term.char_at(0, 0), 'B');
        assert_eq!(term.char_at(1, 0), 'C');
        assert_eq!(term.char_at(2, 0), 'D');
    }

    #[test]
    fn insert_lines_pushes_down() {
        let mut term = Terminal::new(5, 5);
        term.process_str("A\r\nB\r\nC");
        term.process_str("\x1b[1;1H\x1b[2L");
        // Row 0 should be blank (inserted), row 1 blank, row 2 = A
        assert!(term.grid.cell(0, 0).unwrap().ch == ' ');
        assert_eq!(term.char_at(2, 0), 'A');
    }

    #[test]
    fn delete_lines_pulls_up() {
        let mut term = Terminal::new(5, 5);
        term.process_str("A\r\nB\r\nC");
        term.process_str("\x1b[1;1H\x1b[1M");
        // Row 0 should now be B, row 1 = C
        assert_eq!(term.char_at(0, 0), 'B');
        assert_eq!(term.char_at(1, 0), 'C');
    }

    #[test]
    fn insert_chars_shifts_right() {
        let mut term = Terminal::new(5, 10);
        term.process_str("ABC");
        term.process_str("\x1b[1;1H\x1b[2@");
        // Original chars should be shifted right by 2
        assert_eq!(term.char_at(0, 0), ' ');
        assert_eq!(term.char_at(0, 1), ' ');
        assert_eq!(term.char_at(0, 2), 'A');
    }

    #[test]
    fn delete_chars_shifts_left() {
        let mut term = Terminal::new(5, 10);
        term.process_str("ABCDE");
        term.process_str("\x1b[1;1H\x1b[2P");
        // First 2 chars deleted, C moves to col 0
        assert_eq!(term.char_at(0, 0), 'C');
        assert_eq!(term.char_at(0, 1), 'D');
        assert_eq!(term.char_at(0, 2), 'E');
    }

    #[test]
    fn tab_moves_to_next_tab_stop() {
        let mut term = Terminal::new(5, 80);
        term.process_str("\t");
        assert_eq!(term.grid.cursor().col, 8);
    }

    #[test]
    fn random_bytes_dont_panic() {
        let mut term = Terminal::new(10, 80);
        // Feed various random-ish byte sequences
        let sequences: Vec<Vec<u8>> = vec![
            vec![0x1b, b'[', b'1', b';', b'2', b'H'],
            vec![0x1b, b'[', b'm'],
            vec![0x1b, b'[', b'0', b'J'],
            vec![0x1b, b'[', b'?', b'2', b'5', b'h'], // private mode
            vec![0x1b, b']', b'0', b';', b't', b'i', b't', b'l', b'e', 0x07], // OSC
            vec![0xff, 0xfe, 0xfd],                   // invalid UTF-8
            vec![0x1b, b'[', b'2', b'0', b'0', b'~'], // bracketed paste
            (0..=255u8).collect::<Vec<_>>(),          // all byte values
        ];
        for seq in &sequences {
            term.process(seq);
        }
    }
}
