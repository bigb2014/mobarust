//! Tab manager: multiple terminal sessions in tabs.
//!
//! Each tab owns a PTY session and a terminal engine. The tab manager
//! tracks the active tab and provides create/switch/close operations.

use moba_term::pty::PtySession;
use moba_term::Terminal;

/// A single terminal tab with its own PTY and terminal engine.
pub struct TerminalTab {
    /// Unique identifier for this tab.
    pub id: usize,
    /// Display label shown on the tab.
    pub label: String,
    /// The terminal engine (VT parser + grid).
    pub terminal: Terminal,
    /// The PTY session (None if the tab is not connected).
    pub pty: Option<PtySession>,
    /// Input buffer for bytes typed by the user.
    pub input_buf: Vec<u8>,
    /// Whether the terminal process is still alive.
    pub alive: bool,
}

impl TerminalTab {
    /// Creates a new tab with a local PTY shell.
    ///
    /// # Errors
    /// Returns an error string if the PTY cannot be spawned.
    pub fn new_local(id: usize, label: &str, rows: usize, cols: usize) -> Result<Self, String> {
        let terminal = Terminal::new(rows, cols);
        let pty = PtySession::new(rows, cols).map_err(|e| format!("pty spawn failed: {e}"))?;
        Ok(Self {
            id,
            label: label.to_string(),
            terminal,
            pty: Some(pty),
            input_buf: Vec::new(),
            alive: true,
        })
    }

    /// Creates a new tab without a PTY (for testing or disconnected tabs).
    pub fn new_disconnected(id: usize, label: &str, rows: usize, cols: usize) -> Self {
        Self {
            id,
            label: label.to_string(),
            terminal: Terminal::new(rows, cols),
            pty: None,
            input_buf: Vec::new(),
            alive: false,
        }
    }

    /// Polls the PTY for output and processes it through the terminal.
    pub fn poll(&mut self) {
        if let Some(ref mut pty) = self.pty {
            if self.alive {
                let mut buf = [0u8; 4096];
                match pty.read(&mut buf) {
                    Ok(0) => self.alive = false,
                    Ok(n) => self.terminal.process(&buf[..n]),
                    Err(ref e) => {
                        if let moba_term::pty::TermError::Io(ref io_err) = e {
                            if io_err.kind() == std::io::ErrorKind::WouldBlock
                                || io_err.kind() == std::io::ErrorKind::TimedOut
                            {
                                return;
                            }
                        }
                        tracing::warn!("tab {} pty read error: {}", self.id, e);
                        self.alive = false;
                    }
                }

                if !self.input_buf.is_empty() {
                    if let Err(e) = pty.write(&self.input_buf) {
                        tracing::warn!("tab {} pty write error: {}", self.id, e);
                    }
                    self.input_buf.clear();
                }

                if !pty.is_alive() {
                    self.alive = false;
                }
            }
        }
    }

    /// Queues input bytes to be sent to the PTY.
    pub fn send_input(&mut self, bytes: &[u8]) {
        self.input_buf.extend_from_slice(bytes);
    }

    /// Kills the PTY child process if alive.
    pub fn kill(&mut self) {
        if let Some(ref mut pty) = self.pty {
            if self.alive {
                let _ = pty.kill();
                self.alive = false;
            }
        }
    }
}

/// Manages multiple terminal tabs.
pub struct TabManager {
    /// All open tabs.
    tabs: Vec<TerminalTab>,
    /// ID of the currently active tab (index into tabs).
    active: usize,
    /// Next tab ID to assign.
    next_id: usize,
    /// Terminal dimensions shared across tabs.
    rows: usize,
    cols: usize,
}

impl TabManager {
    /// Creates a new tab manager with the given terminal dimensions.
    #[must_use]
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
            next_id: 0,
            rows,
            cols,
        }
    }

    /// Returns the number of open tabs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// Returns true if there are no open tabs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// Returns the index of the active tab, or None if no tabs are open.
    #[must_use]
    pub fn active_index(&self) -> Option<usize> {
        if self.tabs.is_empty() {
            None
        } else {
            Some(self.active.min(self.tabs.len() - 1))
        }
    }

    /// Returns a reference to the active tab, if any.
    #[must_use]
    pub fn active_tab(&self) -> Option<&TerminalTab> {
        self.active_index().and_then(|i| self.tabs.get(i))
    }

    /// Returns a mutable reference to the active tab, if any.
    pub fn active_tab_mut(&mut self) -> Option<&mut TerminalTab> {
        self.active_index().and_then(move |i| self.tabs.get_mut(i))
    }

    /// Returns a reference to all tabs.
    #[must_use]
    pub fn tabs(&self) -> &[TerminalTab] {
        &self.tabs
    }

    /// Returns mutable references to all tabs.
    pub fn tabs_mut(&mut self) -> &mut [TerminalTab] {
        &mut self.tabs
    }

    /// Creates a new tab with a local shell and makes it active.
    ///
    /// # Errors
    /// Returns an error string if the PTY cannot be spawned.
    pub fn new_tab(&mut self, label: Option<&str>) -> Result<usize, String> {
        let id = self.next_id;
        self.next_id += 1;
        let label = label.unwrap_or("New Tab");
        let tab = TerminalTab::new_local(id, label, self.rows, self.cols)?;
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
        Ok(id)
    }

    /// Creates a new disconnected tab (for testing).
    pub fn new_disconnected_tab(&mut self, label: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let tab = TerminalTab::new_disconnected(id, label, self.rows, self.cols);
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
        id
    }

    /// Switches to the tab at the given index.
    pub fn switch_to(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active = index;
        }
    }

    /// Closes the tab at the given index.
    /// If the closed tab was active, switches to the previous tab.
    pub fn close_tab(&mut self, index: usize) {
        if index >= self.tabs.len() {
            return;
        }
        if let Some(tab) = self.tabs.get_mut(index) {
            tab.kill();
        }
        self.tabs.remove(index);
        if self.active >= self.tabs.len() && !self.tabs.is_empty() {
            self.active = self.tabs.len() - 1;
        } else if self.tabs.is_empty() {
            self.active = 0;
        }
    }

    /// Polls all tabs for PTY output.
    pub fn poll_all(&mut self) {
        for tab in &mut self.tabs {
            tab.poll();
        }
    }

    /// Resizes all tabs to new dimensions.
    pub fn resize_all(&mut self, rows: usize, cols: usize) {
        self.rows = rows;
        self.cols = cols;
        for tab in &mut self.tabs {
            tab.terminal.grid.resize(rows, cols);
            if let Some(ref mut pty) = tab.pty {
                let _ = pty.resize(rows, cols);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tab_manager_is_empty() {
        let mgr = TabManager::new(24, 80);
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        assert!(mgr.active_tab().is_none());
    }

    #[test]
    fn new_disconnected_tab_is_added() {
        let mut mgr = TabManager::new(24, 80);
        let id = mgr.new_disconnected_tab("Test");
        assert_eq!(mgr.len(), 1);
        assert_eq!(id, 0);
        assert!(mgr.active_tab().is_some());
        assert_eq!(mgr.active_tab().unwrap().label, "Test");
    }

    #[test]
    fn multiple_tabs_switch() {
        let mut mgr = TabManager::new(24, 80);
        mgr.new_disconnected_tab("Tab A");
        mgr.new_disconnected_tab("Tab B");
        mgr.new_disconnected_tab("Tab C");
        assert_eq!(mgr.len(), 3);
        assert_eq!(mgr.active_tab().unwrap().label, "Tab C");

        mgr.switch_to(0);
        assert_eq!(mgr.active_tab().unwrap().label, "Tab A");

        mgr.switch_to(1);
        assert_eq!(mgr.active_tab().unwrap().label, "Tab B");
    }

    #[test]
    fn close_tab() {
        let mut mgr = TabManager::new(24, 80);
        mgr.new_disconnected_tab("A");
        mgr.new_disconnected_tab("B");
        mgr.new_disconnected_tab("C");
        assert_eq!(mgr.len(), 3);

        mgr.close_tab(1);
        assert_eq!(mgr.len(), 2);
        assert_eq!(mgr.tabs()[0].label, "A");
        assert_eq!(mgr.tabs()[1].label, "C");
    }

    #[test]
    fn close_active_tab_switches_to_previous() {
        let mut mgr = TabManager::new(24, 80);
        mgr.new_disconnected_tab("A");
        mgr.new_disconnected_tab("B");
        mgr.new_disconnected_tab("C");
        // Active is at index 2 ("C")
        mgr.close_tab(2);
        assert_eq!(mgr.active_tab().unwrap().label, "B");
    }

    #[test]
    fn close_all_tabs() {
        let mut mgr = TabManager::new(24, 80);
        mgr.new_disconnected_tab("A");
        mgr.new_disconnected_tab("B");
        mgr.close_tab(0);
        mgr.close_tab(0);
        assert!(mgr.is_empty());
        assert!(mgr.active_tab().is_none());
    }

    #[test]
    fn switch_out_of_bounds_does_nothing() {
        let mut mgr = TabManager::new(24, 80);
        mgr.new_disconnected_tab("A");
        mgr.switch_to(99);
        assert_eq!(mgr.active_tab().unwrap().label, "A");
    }

    #[test]
    fn close_out_of_bounds_does_nothing() {
        let mut mgr = TabManager::new(24, 80);
        mgr.new_disconnected_tab("A");
        mgr.close_tab(99);
        assert_eq!(mgr.len(), 1);
    }

    #[test]
    fn tab_ids_are_unique() {
        let mut mgr = TabManager::new(24, 80);
        let id1 = mgr.new_disconnected_tab("A");
        let id2 = mgr.new_disconnected_tab("B");
        mgr.close_tab(0);
        let id3 = mgr.new_disconnected_tab("C");
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
    }

    #[test]
    fn poll_all_doesnt_panic() {
        let mut mgr = TabManager::new(24, 80);
        mgr.new_disconnected_tab("A");
        mgr.new_disconnected_tab("B");
        mgr.poll_all();
    }
}
