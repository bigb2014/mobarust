//! MobaRust application: single-tab local terminal.
//!
//! Ties together the PTY session, terminal engine, and egui renderer
//! into a working terminal application.

use eframe::egui;
use moba_term::pty::PtySession;
use moba_term::Terminal;

/// The MobaRust application state.
pub struct MobaApp {
    /// The terminal engine (VT parser + grid).
    terminal: Terminal,
    /// The PTY session.
    pty: Option<PtySession>,
    /// Buffer of input bytes to send to PTY.
    input_buf: Vec<u8>,
    /// Whether the terminal is still alive.
    alive: bool,
}

impl MobaApp {
    /// Creates a new MobaRust app with a local PTY shell.
    ///
    /// # Errors
    /// Returns an error if the PTY cannot be spawned.
    pub fn new(rows: usize, cols: usize) -> Result<Self, String> {
        let terminal = Terminal::new(rows, cols);
        let pty = PtySession::new(rows, cols).map_err(|e| format!("pty spawn failed: {e}"))?;

        Ok(Self {
            terminal,
            pty: Some(pty),
            input_buf: Vec::new(),
            alive: true,
        })
    }
}

impl eframe::App for MobaApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Process PTY output and input in the logic pass.
        if let Some(ref mut pty) = self.pty {
            if self.alive {
                // Try to read from the PTY.
                let mut buf = [0u8; 4096];
                match pty.read(&mut buf) {
                    Ok(0) => {
                        self.alive = false;
                    }
                    Ok(n) => {
                        self.terminal.process(&buf[..n]);
                    }
                    Err(ref e) => {
                        // TermError wraps io::Error; check the inner error.
                        if let moba_term::pty::TermError::Io(ref io_err) = e {
                            if io_err.kind() == std::io::ErrorKind::WouldBlock
                                || io_err.kind() == std::io::ErrorKind::TimedOut
                            {
                                // Non-blocking read with no data, fine.
                            }
                        } else {
                            tracing::warn!("pty read error: {e}");
                            self.alive = false;
                        }
                    }
                }

                // Send any pending input.
                if !self.input_buf.is_empty() {
                    if let Err(e) = pty.write(&self.input_buf) {
                        tracing::warn!("pty write error: {e}");
                    }
                    self.input_buf.clear();
                }

                if !pty.is_alive() {
                    self.alive = false;
                }
            }
        }

        if self.alive {
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let config = crate::term_view::TermViewConfig::default();
        let mut local_input: Vec<u8> = Vec::new();
        let input_ref = &mut local_input;
        let term_view =
            crate::term_view::TermView::new(&mut self.terminal, config, move |bytes: &[u8]| {
                input_ref.extend_from_slice(bytes);
            });
        term_view.show(ui);
        if !local_input.is_empty() {
            self.input_buf.extend_from_slice(&local_input);
        }

        if !self.alive {
            ui.label(
                egui::RichText::new("[process exited]")
                    .color(egui::Color32::from_rgb(0xff, 0x80, 0x80)),
            );
        }
    }
}
