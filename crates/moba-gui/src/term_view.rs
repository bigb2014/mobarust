//! Terminal view: renders the moba-term grid in an egui UI.
//!
//! This widget draws the terminal grid character-by-character using the
//! egui painter, handles keyboard input, and manages scrollback display.

use egui::{Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};
use moba_term::grid::{Attributes, Cell, Color};
use moba_term::Terminal;

/// ANSI 16-color palette mapped to egui Color32 values.
const ANSI_COLORS: [Color32; 16] = [
    Color32::from_rgb(0x00, 0x00, 0x00), // 0: black
    Color32::from_rgb(0xc0, 0x00, 0x00), // 1: red
    Color32::from_rgb(0x00, 0xc0, 0x00), // 2: green
    Color32::from_rgb(0xc0, 0xc0, 0x00), // 3: yellow
    Color32::from_rgb(0x00, 0x00, 0xc0), // 4: blue
    Color32::from_rgb(0xc0, 0x00, 0xc0), // 5: magenta
    Color32::from_rgb(0x00, 0xc0, 0xc0), // 6: cyan
    Color32::from_rgb(0xc0, 0xc0, 0xc0), // 7: white (default fg)
    Color32::from_rgb(0x80, 0x80, 0x80), // 8: bright black (gray)
    Color32::from_rgb(0xff, 0x00, 0x00), // 9: bright red
    Color32::from_rgb(0x00, 0xff, 0x00), // 10: bright green
    Color32::from_rgb(0xff, 0xff, 0x00), // 11: bright yellow
    Color32::from_rgb(0x00, 0x00, 0xff), // 12: bright blue
    Color32::from_rgb(0xff, 0x00, 0xff), // 13: bright magenta
    Color32::from_rgb(0x00, 0xff, 0xff), // 14: bright cyan
    Color32::from_rgb(0xff, 0xff, 0xff), // 15: bright white
];

/// Maps an ANSI Color index to an egui Color32.
fn color_to_color32(c: Color) -> Color32 {
    ANSI_COLORS[(c.0 as usize) % 16]
}

/// Maps cell Attributes to a (fg, bg) Color32 pair.
fn attrs_to_colors(attrs: &Attributes) -> (Color32, Color32) {
    let fg = color_to_color32(attrs.fg);
    let bg = color_to_color32(attrs.bg);
    if attrs.reverse {
        (bg, fg)
    } else {
        (fg, bg)
    }
}

/// Configuration for the terminal view.
#[derive(Clone, Debug)]
pub struct TermViewConfig {
    /// Font size in pixels.
    pub font_size: f32,
    /// Number of rows in the terminal.
    pub rows: usize,
    /// Number of columns in the terminal.
    pub cols: usize,
}

impl Default for TermViewConfig {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            rows: 24,
            cols: 80,
        }
    }
}

/// Callback type for sending user input bytes to the PTY.
pub type InputHandler<'a> = Box<dyn FnMut(&[u8]) + 'a>;

/// A terminal view widget that renders a `Terminal` in an egui UI.
///
/// Draws the grid character-by-character and captures keyboard input
/// for the terminal. The caller is responsible for feeding input bytes
/// to the terminal and reading output from the PTY.
pub struct TermView<'a> {
    /// The terminal to render.
    pub terminal: &'a mut Terminal,
    /// Configuration (font size, dimensions).
    pub config: TermViewConfig,
    /// Callback for bytes typed by the user (sent to PTY).
    pub input_handler: InputHandler<'a>,
}

impl<'a> TermView<'a> {
    /// Creates a new terminal view.
    pub fn new<F>(terminal: &'a mut Terminal, config: TermViewConfig, input_handler: F) -> Self
    where
        F: FnMut(&[u8]) + 'a,
    {
        Self {
            terminal,
            config,
            input_handler: Box::new(input_handler),
        }
    }

    /// Renders the terminal view and returns the egui response.
    pub fn show(self, ui: &mut Ui) -> Response {
        let font_id = FontId::monospace(self.config.font_size);
        let char_w = self.config.font_size * 0.6;
        let char_h = self.config.font_size * 1.2;
        let grid_w = self.config.cols as f32 * char_w;
        let grid_h = self.config.rows as f32 * char_h;

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(grid_w, grid_h), Sense::CLICK | Sense::FOCUSABLE);

        // Draw background.
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, Color32::from_rgb(0x12, 0x12, 0x12));

        // Draw cells.
        for row in 0..self.config.rows {
            for col in 0..self.config.cols {
                let cell: Cell = self
                    .terminal
                    .grid
                    .cell(row, col)
                    .copied()
                    .unwrap_or_default();
                if cell.ch == ' ' {
                    // Draw background only.
                    let (fg, bg) = attrs_to_colors(&cell.attrs);
                    if bg != Color32::from_rgb(0x12, 0x12, 0x12) {
                        let cell_rect = cell_rect(rect, row, col, char_w, char_h);
                        painter.rect_filled(cell_rect, 0.0, bg);
                    }
                    let _ = fg; // suppress unused warning
                } else {
                    let (fg, bg) = attrs_to_colors(&cell.attrs);
                    let pos = Pos2::new(
                        rect.left() + col as f32 * char_w,
                        rect.top() + row as f32 * char_h,
                    );
                    if bg != Color32::from_rgb(0x12, 0x12, 0x12) {
                        let cell_rect = cell_rect(rect, row, col, char_w, char_h);
                        painter.rect_filled(cell_rect, 0.0, bg);
                    }
                    let _ = painter.text(
                        pos,
                        egui::Align2::LEFT_TOP,
                        cell.ch.to_string(),
                        font_id.clone(),
                        fg,
                    );
                }
            }
        }

        // Draw cursor as a block outline.
        let cursor = self.terminal.grid.cursor();
        if cursor.row < self.config.rows && cursor.col < self.config.cols {
            let cur_rect = cell_rect(rect, cursor.row, cursor.col, char_w, char_h);
            painter.rect_stroke(
                cur_rect,
                0.0,
                Stroke::new(1.0, Color32::from_rgb(0xff, 0xff, 0xff)),
                egui::StrokeKind::Inside,
            );
        }

        // Handle keyboard input.
        let mut input_handler = self.input_handler;
        if response.has_focus() {
            ui.input(|inp| {
                for event in &inp.events {
                    if let egui::Event::Text(s) = event {
                        (input_handler)(s.as_bytes());
                    }
                    if let egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } = event
                    {
                        let bytes = key_to_bytes(*key, *modifiers);
                        if !bytes.is_empty() {
                            (input_handler)(&bytes);
                        }
                    }
                }
            });
        }

        response
    }
}

/// Computes the rect for a single cell.
fn cell_rect(origin: Rect, row: usize, col: usize, char_w: f32, char_h: f32) -> Rect {
    let x = origin.left() + col as f32 * char_w;
    let y = origin.top() + row as f32 * char_h;
    Rect::from_min_size(Pos2::new(x, y), Vec2::new(char_w, char_h))
}

/// Maps an egui key press to terminal byte sequences.
fn key_to_bytes(key: egui::Key, modifiers: egui::Modifiers) -> Vec<u8> {
    if modifiers.ctrl {
        // Ctrl+letter = control character
        match key {
            egui::Key::A => vec![0x01],
            egui::Key::B => vec![0x02],
            egui::Key::C => vec![0x03],
            egui::Key::D => vec![0x04],
            egui::Key::E => vec![0x05],
            egui::Key::F => vec![0x06],
            egui::Key::G => vec![0x07],
            egui::Key::H => vec![0x08],
            egui::Key::I => vec![0x09],
            egui::Key::J => vec![0x0a],
            egui::Key::K => vec![0x0b],
            egui::Key::L => vec![0x0c],
            egui::Key::M => vec![0x0d],
            egui::Key::N => vec![0x0e],
            egui::Key::O => vec![0x0f],
            egui::Key::P => vec![0x10],
            egui::Key::Q => vec![0x11],
            egui::Key::R => vec![0x12],
            egui::Key::S => vec![0x13],
            egui::Key::T => vec![0x14],
            egui::Key::U => vec![0x15],
            egui::Key::V => vec![0x16],
            egui::Key::W => vec![0x17],
            egui::Key::X => vec![0x18],
            egui::Key::Y => vec![0x19],
            egui::Key::Z => vec![0x1a],
            _ => vec![],
        }
    } else {
        match key {
            egui::Key::ArrowUp => vec![0x1b, b'[', b'A'],
            egui::Key::ArrowDown => vec![0x1b, b'[', b'B'],
            egui::Key::ArrowRight => vec![0x1b, b'[', b'C'],
            egui::Key::ArrowLeft => vec![0x1b, b'[', b'D'],
            egui::Key::Home => vec![0x1b, b'[', b'H'],
            egui::Key::End => vec![0x1b, b'[', b'F'],
            egui::Key::Delete => vec![0x1b, b'[', b'3', b'~'],
            egui::Key::Tab => vec![0x09],
            egui::Key::Enter => vec![0x0d],
            egui::Key::Backspace => vec![0x08],
            egui::Key::Escape => vec![0x1b],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_mapping_basic() {
        assert_eq!(
            color_to_color32(Color::BLACK),
            Color32::from_rgb(0x00, 0x00, 0x00)
        );
        assert_eq!(
            color_to_color32(Color::RED),
            Color32::from_rgb(0xc0, 0x00, 0x00)
        );
        assert_eq!(
            color_to_color32(Color::GREEN),
            Color32::from_rgb(0x00, 0xc0, 0x00)
        );
    }

    #[test]
    fn attrs_to_colors_normal() {
        let attrs = Attributes {
            bold: false,
            italic: false,
            underline: false,
            reverse: false,
            fg: Color::RED,
            bg: Color::BLACK,
        };
        let (fg, bg) = attrs_to_colors(&attrs);
        assert_eq!(fg, Color32::from_rgb(0xc0, 0x00, 0x00));
        assert_eq!(bg, Color32::from_rgb(0x00, 0x00, 0x00));
    }

    #[test]
    fn attrs_to_colors_reverse() {
        let attrs = Attributes {
            bold: false,
            italic: false,
            underline: false,
            reverse: true,
            fg: Color::RED,
            bg: Color::GREEN,
        };
        let (fg, bg) = attrs_to_colors(&attrs);
        // Reverse swaps fg and bg.
        assert_eq!(fg, Color32::from_rgb(0x00, 0xc0, 0x00)); // bg (green)
        assert_eq!(bg, Color32::from_rgb(0xc0, 0x00, 0x00)); // fg (red)
    }

    #[test]
    fn key_to_bytes_arrow_keys() {
        assert_eq!(
            key_to_bytes(egui::Key::ArrowUp, egui::Modifiers::default()),
            vec![0x1b, b'[', b'A']
        );
        assert_eq!(
            key_to_bytes(egui::Key::ArrowDown, egui::Modifiers::default()),
            vec![0x1b, b'[', b'B']
        );
    }

    #[test]
    fn key_to_bytes_ctrl_c() {
        let mods = egui::Modifiers {
            ctrl: true,
            ..Default::default()
        };
        assert_eq!(key_to_bytes(egui::Key::C, mods), vec![0x03]);
    }

    #[test]
    fn key_to_bytes_enter() {
        assert_eq!(
            key_to_bytes(egui::Key::Enter, egui::Modifiers::default()),
            vec![0x0d]
        );
    }

    #[test]
    fn cell_rect_computation() {
        let origin = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(100.0, 100.0));
        let r = cell_rect(origin, 2, 3, 5.0, 10.0);
        assert_eq!(r.left(), 10.0 + 3.0 * 5.0);
        assert_eq!(r.top(), 20.0 + 2.0 * 10.0);
    }
}
