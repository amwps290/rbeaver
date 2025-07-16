use crate::database::{ConnectionParams, DatabaseType, SslMode};
use egui::{ComboBox, TextEdit, Ui};

/// Connection dialog for database connections
#[derive(Default)]
pub struct ConnectionDialog {
    params: ConnectionParams,
    original_params: Option<ConnectionParams>, // For editing mode
    show_advanced: bool,
    test_result: Option<Result<(), String>>,
    is_testing: bool,
    is_editing: bool,
    validation_errors: Vec<String>,
}

impl ConnectionDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_params(params: ConnectionParams) -> Self {
        Self {
            params,
            original_params: None,
            show_advanced: false,
            test_result: None,
            is_testing: false,
            is_editing: false,
            validation_errors: Vec::new(),
        }
    }

    /// Create dialog for editing an existing connection
    pub fn for_editing(params: ConnectionParams) -> Self {
        Self {
            params: params.clone(),
            original_params: Some(params),
            show_advanced: false,
            test_result: None,
            is_testing: false,
            is_editing: true,
            validation_errors: Vec::new(),
        }
    }

    /// Check if the dialog is in editing mode
    pub fn is_editing(&self) -> bool {
        self.is_editing
    }

    /// Validate the current parameters
    fn validate(&mut self) -> bool {
        self.validation_errors.clear();

        match self.params.validate() {
            Ok(()) => true,
            Err(error) => {
                self.validation_errors.push(error);
                false
            }
        }
    }

    pub fn render(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            let title = if self.is_editing {
                format!("Edit Connection: {}", self.params.name)
            } else {
                "New Database Connection".to_string()
            };
            ui.heading(title);
            ui.separator();

            // Show validation errors
            if !self.validation_errors.is_empty() {
                ui.colored_label(egui::Color32::RED, "⚠️ Validation Errors:");
                for error in &self.validation_errors {
                    ui.colored_label(egui::Color32::RED, format!("• {}", error));
                }
                ui.separator();
            }

            // Connection name
            ui.horizontal(|ui| {
                ui.label("Connection Name:");
                ui.add(TextEdit::singleline(&mut self.params.name).desired_width(200.0));
            });

            ui.add_space(8.0);

            // Database type
            ui.horizontal(|ui| {
                ui.label("Database Type:");
                ComboBox::from_id_source("db_type")
                    .selected_text(format!("{:?}", self.params.database_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.params.database_type,
                            DatabaseType::PostgreSQL,
                            "PostgreSQL",
                        );
                        ui.selectable_value(
                            &mut self.params.database_type,
                            DatabaseType::MySQL,
                            "MySQL",
                        );
                        ui.selectable_value(
                            &mut self.params.database_type,
                            DatabaseType::SQLite,
                            "SQLite",
                        );
                    });
            });

            ui.add_space(8.0);

            // Basic connection parameters
            ui.horizontal(|ui| {
                ui.label("Host:");
                ui.add(TextEdit::singleline(&mut self.params.host).desired_width(150.0));
                ui.label("Port:");
                ui.add(egui::DragValue::new(&mut self.params.port).range(1..=65535));
            });

            ui.horizontal(|ui| {
                ui.label("Database:");
                ui.add(TextEdit::singleline(&mut self.params.database).desired_width(200.0));
            });

            ui.horizontal(|ui| {
                ui.label("Username:");
                ui.add(TextEdit::singleline(&mut self.params.username).desired_width(200.0));
            });

            ui.horizontal(|ui| {
                ui.label("Password:");
                ui.add(
                    TextEdit::singleline(&mut self.params.password)
                        .password(true)
                        .desired_width(200.0),
                );
            });

            ui.add_space(8.0);

            // Advanced settings toggle
            if ui
                .button(if self.show_advanced {
                    "Hide Advanced"
                } else {
                    "Show Advanced"
                })
                .clicked()
            {
                self.show_advanced = !self.show_advanced;
            }

            if self.show_advanced {
                ui.separator();
                ui.heading("Advanced Settings");

                // SSL Mode
                ui.horizontal(|ui| {
                    ui.label("SSL Mode:");
                    ComboBox::from_id_source("ssl_mode")
                        .selected_text(format!("{:?}", self.params.ssl_mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.params.ssl_mode,
                                SslMode::Disable,
                                "Disable",
                            );
                            ui.selectable_value(&mut self.params.ssl_mode, SslMode::Allow, "Allow");
                            ui.selectable_value(
                                &mut self.params.ssl_mode,
                                SslMode::Prefer,
                                "Prefer",
                            );
                            ui.selectable_value(
                                &mut self.params.ssl_mode,
                                SslMode::Require,
                                "Require",
                            );
                            ui.selectable_value(
                                &mut self.params.ssl_mode,
                                SslMode::VerifyCa,
                                "Verify CA",
                            );
                            ui.selectable_value(
                                &mut self.params.ssl_mode,
                                SslMode::VerifyFull,
                                "Verify Full",
                            );
                        });
                });

                // Connection timeout
                ui.horizontal(|ui| {
                    ui.label("Connection Timeout (seconds):");
                    let mut timeout = self.params.connection_timeout.unwrap_or(30);
                    ui.add(egui::DragValue::new(&mut timeout).range(1..=300));
                    self.params.connection_timeout = Some(timeout);
                });
            }

            ui.add_space(12.0);

            // Test connection result
            if let Some(result) = &self.test_result {
                match result {
                    Ok(()) => {
                        ui.colored_label(
                            egui::Color32::from_rgb(0, 128, 0),
                            "✓ Connection successful!",
                        );
                    }
                    Err(error) => {
                        ui.colored_label(
                            egui::Color32::from_rgb(220, 53, 69),
                            format!("✗ Connection failed: {}", error),
                        );
                    }
                }
                ui.add_space(8.0);
            }

            if self.is_testing {
                ui.spinner();
                ui.label("Testing connection...");
            }
        });
    }

    pub fn get_params(&self) -> &ConnectionParams {
        &self.params
    }

    pub fn get_params_mut(&mut self) -> &mut ConnectionParams {
        &mut self.params
    }

    pub fn set_test_result(&mut self, result: Result<(), String>) {
        self.test_result = Some(result);
        self.is_testing = false;
    }

    pub fn set_testing(&mut self, testing: bool) {
        self.is_testing = testing;
        if testing {
            self.test_result = None;
        }
    }

    pub fn reset(&mut self) {
        self.params = ConnectionParams::default();
        self.original_params = None;
        self.show_advanced = false;
        self.test_result = None;
        self.is_testing = false;
        self.is_editing = false;
        self.validation_errors.clear();
    }

    /// Render action buttons and return the action taken
    pub fn render_buttons(&mut self, ui: &mut Ui) -> DialogAction {
        let mut action = DialogAction::None;

        ui.horizontal(|ui| {
            // Test Connection button
            if ui.button("Test Connection").clicked() && !self.is_testing {
                if self.validate() {
                    action = DialogAction::TestConnection;
                }
            }

            ui.add_space(10.0);

            // Save/Update button
            let save_text = if self.is_editing {
                "Update"
            } else {
                "Save & Connect"
            };
            if ui.button(save_text).clicked() {
                if self.validate() {
                    action = if self.is_editing {
                        DialogAction::UpdateConnection
                    } else {
                        DialogAction::SaveAndConnect
                    };
                }
            }

            // Save Only button (for new connections)
            if !self.is_editing {
                if ui.button("Save Only").clicked() {
                    if self.validate() {
                        action = DialogAction::SaveOnly;
                    }
                }
            }

            ui.add_space(10.0);

            // Cancel button
            if ui.button("Cancel").clicked() {
                action = DialogAction::Cancel;
            }
        });

        action
    }

    /// Check if connection parameters have changed (for editing mode)
    pub fn has_changes(&self) -> bool {
        if let Some(original) = &self.original_params {
            // Compare relevant fields (excluding ID)
            original.name != self.params.name
                || original.database_type != self.params.database_type
                || original.host != self.params.host
                || original.port != self.params.port
                || original.database != self.params.database
                || original.username != self.params.username
                || original.password != self.params.password
                || original.ssl_mode != self.params.ssl_mode
                || original.connection_timeout != self.params.connection_timeout
        } else {
            true // New connection always has "changes"
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DialogAction {
    None,
    TestConnection,
    SaveAndConnect,
    SaveOnly,
    UpdateConnection,
    Cancel,
}
