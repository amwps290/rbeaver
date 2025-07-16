use egui::Ui;

/// Confirmation dialog for destructive operations
#[derive(Debug, Clone)]
pub struct ConfirmationDialog {
    pub show: bool,
    pub title: String,
    pub message: String,
    pub confirm_text: String,
    pub cancel_text: String,
    pub confirmed: bool,
    pub cancelled: bool,
}

impl Default for ConfirmationDialog {
    fn default() -> Self {
        Self {
            show: false,
            title: "Confirm".to_string(),
            message: "Are you sure?".to_string(),
            confirm_text: "Yes".to_string(),
            cancel_text: "Cancel".to_string(),
            confirmed: false,
            cancelled: false,
        }
    }
}

impl ConfirmationDialog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Show a delete confirmation dialog
    pub fn show_delete_confirmation(&mut self, item_name: &str) {
        self.show = true;
        self.title = "Delete Connection".to_string();
        self.message = format!(
            "Are you sure you want to delete the connection '{}'?\n\nThis action cannot be undone.",
            item_name
        );
        self.confirm_text = "Delete".to_string();
        self.cancel_text = "Cancel".to_string();
        self.confirmed = false;
        self.cancelled = false;
    }

    /// Show a generic confirmation dialog
    pub fn show_confirmation(&mut self, title: &str, message: &str, confirm_text: &str) {
        self.show = true;
        self.title = title.to_string();
        self.message = message.to_string();
        self.confirm_text = confirm_text.to_string();
        self.cancel_text = "Cancel".to_string();
        self.confirmed = false;
        self.cancelled = false;
    }

    /// Render the confirmation dialog
    pub fn render(&mut self, ctx: &egui::Context) {
        if !self.show {
            return;
        }

        egui::Window::new(&self.title)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_min_width(300.0);

                // Message
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);

                    // Icon based on dialog type
                    if self.title.contains("Delete") {
                        ui.label(egui::RichText::new("⚠️").size(32.0));
                    } else {
                        ui.label(egui::RichText::new("❓").size(32.0));
                    }

                    ui.add_space(10.0);

                    // Message text
                    ui.label(&self.message);

                    ui.add_space(20.0);

                    // Buttons
                    ui.horizontal(|ui| {
                        ui.add_space(50.0);

                        // Confirm button (styled based on action)
                        let confirm_button = if self.title.contains("Delete") {
                            egui::Button::new(&self.confirm_text)
                                .fill(egui::Color32::from_rgb(220, 53, 69)) // Red for delete
                        } else {
                            egui::Button::new(&self.confirm_text)
                                .fill(egui::Color32::from_rgb(40, 167, 69)) // Green for confirm
                        };

                        if ui.add(confirm_button).clicked() {
                            self.confirmed = true;
                            self.show = false;
                        }

                        ui.add_space(10.0);

                        // Cancel button
                        if ui.button(&self.cancel_text).clicked() {
                            self.cancelled = true;
                            self.show = false;
                        }
                    });

                    ui.add_space(10.0);
                });
            });
    }

    /// Check if the dialog was confirmed and reset the state
    pub fn take_confirmed(&mut self) -> bool {
        let confirmed = self.confirmed;
        self.confirmed = false;
        confirmed
    }

    /// Check if the dialog was cancelled and reset the state
    pub fn take_cancelled(&mut self) -> bool {
        let cancelled = self.cancelled;
        self.cancelled = false;
        cancelled
    }

    /// Close the dialog
    pub fn close(&mut self) {
        self.show = false;
        self.confirmed = false;
        self.cancelled = false;
    }

    /// Check if the dialog is currently showing
    pub fn is_showing(&self) -> bool {
        self.show
    }
}
