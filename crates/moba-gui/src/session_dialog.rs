//! Session dialog: create/edit/delete session configurations.
//!
//! Renders an egui modal window for editing session properties.

/// Fields for a session being created or edited.
#[derive(Clone, Debug, Default)]
pub struct SessionForm {
    /// Session display name.
    pub name: String,
    /// Session type as a string (LocalShell, Ssh, Telnet, etc.).
    pub session_type: String,
    /// Remote host.
    pub host: String,
    /// Remote port (as string for the text field).
    pub port: String,
    /// Username for remote sessions.
    pub username: String,
    /// Custom command.
    pub command: String,
    /// Working directory.
    pub working_dir: String,
    /// Tags (comma-separated).
    pub tags: String,
}

/// The mode of the session dialog.
#[derive(Clone, Debug, PartialEq)]
pub enum DialogMode {
    /// Dialog is hidden.
    Hidden,
    /// Creating a new session.
    Create,
    /// Editing an existing session (stores the id).
    Edit(String),
    /// Confirming deletion of a session (stores the id).
    ConfirmDelete(String),
}

/// State for the session dialog.
pub struct SessionDialog {
    /// Current dialog mode.
    mode: DialogMode,
    /// Form fields being edited.
    form: SessionForm,
}

/// Result of a dialog interaction.
#[derive(Clone, Debug)]
pub enum DialogResult {
    /// User confirmed saving the session.
    Save(SessionForm),
    /// User confirmed deleting a session.
    Delete(String),
    /// User cancelled the dialog.
    Cancel,
}

impl SessionDialog {
    /// Creates a new hidden session dialog.
    #[must_use]
    pub fn new() -> Self {
        Self {
            mode: DialogMode::Hidden,
            form: SessionForm::default(),
        }
    }

    /// Returns true if the dialog is currently visible.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.mode != DialogMode::Hidden
    }

    /// Opens the dialog in create mode.
    pub fn open_create(&mut self) {
        self.form = SessionForm {
            session_type: "LocalShell".to_string(),
            ..Default::default()
        };
        self.mode = DialogMode::Create;
    }

    /// Opens the dialog in edit mode with the given session data.
    pub fn open_edit(
        &mut self,
        id: &str,
        name: &str,
        session_type: &str,
        host: &str,
        port: &str,
        username: &str,
    ) {
        self.form = SessionForm {
            name: name.to_string(),
            session_type: session_type.to_string(),
            host: host.to_string(),
            port: port.to_string(),
            username: username.to_string(),
            ..Default::default()
        };
        self.mode = DialogMode::Edit(id.to_string());
    }

    /// Opens the dialog in delete confirmation mode.
    pub fn open_delete(&mut self, id: &str, name: &str) {
        self.form = SessionForm {
            name: name.to_string(),
            ..Default::default()
        };
        self.mode = DialogMode::ConfirmDelete(id.to_string());
    }

    /// Renders the dialog and returns a result if the user confirmed or cancelled.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<DialogResult> {
        if self.mode == DialogMode::Hidden {
            return None;
        }

        let mut result = None;
        let title = match &self.mode {
            DialogMode::Create => "New Session",
            DialogMode::Edit(_) => "Edit Session",
            DialogMode::ConfirmDelete(_) => "Delete Session",
            DialogMode::Hidden => return None,
        };

        let is_delete = matches!(self.mode, DialogMode::ConfirmDelete(_));

        egui::Window::new(title)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                if is_delete {
                    ui.label(format!(
                        "Are you sure you want to delete '{}'?",
                        self.form.name
                    ));
                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            if let DialogMode::ConfirmDelete(id) = &self.mode {
                                result = Some(DialogResult::Delete(id.clone()));
                            }
                            self.mode = DialogMode::Hidden;
                        }
                        if ui.button("Cancel").clicked() {
                            result = Some(DialogResult::Cancel);
                            self.mode = DialogMode::Hidden;
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.form.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Type:");
                        let types = [
                            "LocalShell",
                            "Ssh",
                            "Telnet",
                            "Rdp",
                            "Vnc",
                            "Serial",
                            "Mosh",
                            "Ftp",
                            "Sftp",
                        ];
                        egui::ComboBox::from_label("")
                            .selected_text(self.form.session_type.as_str())
                            .show_ui(ui, |ui| {
                                for t in types {
                                    ui.selectable_value(
                                        &mut self.form.session_type,
                                        t.to_string(),
                                        t,
                                    );
                                }
                            });
                    });
                    if self.form.session_type != "LocalShell" {
                        ui.horizontal(|ui| {
                            ui.label("Host:");
                            ui.text_edit_singleline(&mut self.form.host);
                            ui.label("Port:");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.form.port).desired_width(60.0),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Username:");
                            ui.text_edit_singleline(&mut self.form.username);
                        });
                    }
                    ui.horizontal(|ui| {
                        ui.label("Command:");
                        ui.text_edit_singleline(&mut self.form.command);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Tags:");
                        ui.text_edit_singleline(&mut self.form.tags);
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            result = Some(DialogResult::Save(self.form.clone()));
                            self.mode = DialogMode::Hidden;
                        }
                        if ui.button("Cancel").clicked() {
                            result = Some(DialogResult::Cancel);
                            self.mode = DialogMode::Hidden;
                        }
                    });
                }
            });

        result
    }
}

impl Default for SessionDialog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_dialog_is_hidden() {
        let d = SessionDialog::new();
        assert!(!d.is_visible());
    }

    #[test]
    fn open_create_sets_mode() {
        let mut d = SessionDialog::new();
        d.open_create();
        assert!(d.is_visible());
        assert_eq!(d.mode, DialogMode::Create);
        assert_eq!(d.form.session_type, "LocalShell");
    }

    #[test]
    fn open_edit_sets_mode_and_form() {
        let mut d = SessionDialog::new();
        d.open_edit("s1", "My Server", "Ssh", "10.0.0.1", "22", "admin");
        assert_eq!(d.mode, DialogMode::Edit("s1".to_string()));
        assert_eq!(d.form.name, "My Server");
        assert_eq!(d.form.host, "10.0.0.1");
    }

    #[test]
    fn open_delete_sets_confirm_mode() {
        let mut d = SessionDialog::new();
        d.open_delete("s1", "My Server");
        assert_eq!(d.mode, DialogMode::ConfirmDelete("s1".to_string()));
    }
}
