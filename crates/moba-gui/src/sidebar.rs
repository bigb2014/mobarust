//! Session tree sidebar: lists saved sessions, click to open.

/// Actions emitted by the sidebar when the user interacts with it.
#[derive(Clone, Debug)]
pub enum SidebarAction {
    /// User clicked to open a session.
    OpenSession(String),
    /// User clicked to edit a session.
    EditSession(String),
    /// User clicked to delete a session.
    DeleteSession(String),
    /// User clicked the "New Session" button.
    NewSession,
}

/// Sidebar state tracking which sessions are displayed.
pub struct Sidebar {
    /// Session labels to display: (id, label, group_name).
    sessions: Vec<(String, String, String)>,
    /// Currently selected session id (if any).
    selected: Option<String>,
    /// Actions queued from the last frame.
    actions: Vec<SidebarAction>,
}

impl Sidebar {
    /// Creates a new empty sidebar.
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            selected: None,
            actions: Vec::new(),
        }
    }

    /// Sets the session list to display.
    pub fn set_sessions(&mut self, sessions: Vec<(String, String, String)>) {
        self.sessions = sessions;
    }

    /// Returns the session id currently selected.
    #[must_use]
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Drains queued actions from the sidebar.
    pub fn drain_actions(&mut self) -> Vec<SidebarAction> {
        std::mem::take(&mut self.actions)
    }

    /// Renders the sidebar as a fixed-width column inside the given Ui.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Use a fixed-width vertical column instead of a Panel.
        egui::ScrollArea::vertical()
            .max_width(200.0)
            .show(ui, |ui| {
                ui.heading("Sessions");
                ui.separator();

                if ui.button("+ New Session").clicked() {
                    self.actions.push(SidebarAction::NewSession);
                }

                ui.separator();

                // Group sessions by group name.
                let mut groups: std::collections::BTreeMap<String, Vec<(String, String)>> =
                    std::collections::BTreeMap::new();
                for (id, label, group) in &self.sessions {
                    groups
                        .entry(group.clone())
                        .or_default()
                        .push((id.clone(), label.clone()));
                }

                for (group_name, sessions) in &groups {
                    ui.label(
                        egui::RichText::new(group_name.as_str())
                            .strong()
                            .color(egui::Color32::from_rgb(0xcc, 0xcc, 0xcc)),
                    );
                    for (id, label) in sessions {
                        let is_selected = self.selected.as_deref() == Some(id.as_str());
                        if ui.selectable_label(is_selected, label).clicked() {
                            self.selected = Some(id.clone());
                            self.actions.push(SidebarAction::OpenSession(id.clone()));
                        }
                    }
                    ui.separator();
                }

                if self.sessions.is_empty() {
                    ui.label(
                        egui::RichText::new(
                            "No saved sessions.\nClick + New Session to create one.",
                        )
                        .weak(),
                    );
                }
            });
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sidebar_is_empty() {
        let mut sb = Sidebar::new();
        assert!(sb.selected().is_none());
        assert!(sb.drain_actions().is_empty());
    }

    #[test]
    fn set_sessions_updates_list() {
        let mut sb = Sidebar::new();
        sb.set_sessions(vec![
            (
                "s1".to_string(),
                "Server 1".to_string(),
                "Production".to_string(),
            ),
            (
                "s2".to_string(),
                "Server 2".to_string(),
                "Production".to_string(),
            ),
            ("s3".to_string(), "Local".to_string(), "Local".to_string()),
        ]);
        assert_eq!(sb.sessions.len(), 3);
    }

    #[test]
    fn drain_actions_clears() {
        let mut sb = Sidebar::new();
        sb.actions.push(SidebarAction::NewSession);
        let actions = sb.drain_actions();
        assert_eq!(actions.len(), 1);
        assert!(sb.drain_actions().is_empty());
    }
}
