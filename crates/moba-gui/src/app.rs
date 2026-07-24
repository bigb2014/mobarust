//! MobaRust application: multi-tab terminal with session sidebar.
//!
//! PTY output is read on a background thread (not the UI thread)
//! so the window stays responsive.

use eframe::egui;

use crate::session_dialog::{DialogResult, SessionDialog};
use crate::sidebar::Sidebar;
use crate::tabs::TabManager;

/// The MobaRust application state.
pub struct MobaApp {
    tabs: TabManager,
    sidebar: Sidebar,
    dialog: SessionDialog,
}

impl MobaApp {
    /// Creates a new MobaRust app with an initial local shell tab.
    ///
    /// # Errors
    /// Returns an error if the initial PTY cannot be spawned.
    pub fn new(rows: usize, cols: usize) -> Result<Self, String> {
        let mut tabs = TabManager::new(rows, cols);
        tabs.new_tab(Some("Local Shell"))?;
        Ok(Self {
            tabs,
            sidebar: Sidebar::new(),
            dialog: SessionDialog::new(),
        })
    }

    /// Creates a new MobaRust app without any tabs.
    #[must_use]
    pub fn new_empty(rows: usize, cols: usize) -> Self {
        Self {
            tabs: TabManager::new(rows, cols),
            sidebar: Sidebar::new(),
            dialog: SessionDialog::new(),
        }
    }

    /// Returns a reference to the tab manager.
    #[must_use]
    pub fn tabs(&self) -> &TabManager {
        &self.tabs
    }
}

impl eframe::App for MobaApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll all tabs — non-blocking (uses try_recv on a channel).
        self.tabs.poll_all();

        // Request repaint to keep the terminal updating.
        if self.tabs.tabs().iter().any(|t| t.alive) {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }

        // Process sidebar actions.
        for action in self.sidebar.drain_actions() {
            match action {
                crate::sidebar::SidebarAction::NewSession => {
                    self.dialog.open_create();
                }
                crate::sidebar::SidebarAction::OpenSession(_) => {
                    // For now, opening a session creates a local shell tab.
                    // SSH connection will be wired in when the session config
                    // is passed through to the tab manager.
                    let _ = self.tabs.new_tab(Some("New Session"));
                }
                crate::sidebar::SidebarAction::EditSession(_) => {
                    self.dialog.open_create();
                }
                crate::sidebar::SidebarAction::DeleteSession(_) => {}
            }
        }

        // Show dialog if open.
        if let Some(result) = self.dialog.show(ctx) {
            match result {
                DialogResult::Save(form) => {
                    // Create a tab with the session name.
                    let label = if form.name.is_empty() {
                        "New Session"
                    } else {
                        &form.name
                    };
                    let _ = self.tabs.new_tab(Some(label));
                }
                DialogResult::Delete(_) => {}
                DialogResult::Cancel => {}
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Layout: horizontal split with sidebar (left) and main area (right).
        ui.horizontal(|ui| {
            // Left: sidebar (fixed width 200px).
            ui.allocate_ui_with_layout(
                egui::Vec2::new(200.0, ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    self.sidebar.show(ui);
                },
            );

            // Separator.
            ui.separator();

            // Right: tab bar + terminal.
            ui.vertical(|ui| {
                // Tab bar.
                let mut close_index: Option<usize> = None;
                let mut switch_index: Option<usize> = None;
                let mut add_tab = false;

                ui.horizontal(|ui| {
                    for (i, tab) in self.tabs.tabs().iter().enumerate() {
                        let is_active = self.tabs.active_index() == Some(i);
                        let label = format!("{} {}", if is_active { ">" } else { " " }, tab.label);
                        if ui.selectable_label(is_active, &label).clicked() {
                            switch_index = Some(i);
                        }
                        if ui.button("x").clicked() {
                            close_index = Some(i);
                        }
                    }
                    if ui.button("+").clicked() {
                        add_tab = true;
                    }
                });

                ui.separator();

                // Apply tab actions.
                if let Some(i) = switch_index {
                    self.tabs.switch_to(i);
                }
                if let Some(i) = close_index {
                    self.tabs.close_tab(i);
                }
                if add_tab {
                    let _ = self.tabs.new_tab(Some("New Tab"));
                }

                // Terminal view.
                if let Some(tab_index) = self.tabs.active_index() {
                    if let Some(tab) = self.tabs.tabs_mut().get_mut(tab_index) {
                        let config = crate::term_view::TermViewConfig::default();
                        let mut local_input: Vec<u8> = Vec::new();
                        let input_ref = &mut local_input;
                        let term_view = crate::term_view::TermView::new(
                            &mut tab.terminal,
                            config,
                            move |bytes: &[u8]| {
                                input_ref.extend_from_slice(bytes);
                            },
                        );
                        term_view.show(ui);

                        if !local_input.is_empty() {
                            tab.send_input(&local_input);
                        }

                        if !tab.alive {
                            ui.label(
                                egui::RichText::new("[process exited]")
                                    .color(egui::Color32::from_rgb(0xff, 0x80, 0x80)),
                            );
                        }
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.label("No tabs open. Click + to open a new tab.");
                    });
                }
            });
        });
    }
}
