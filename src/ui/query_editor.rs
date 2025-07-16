use crate::ui::theme::get_sql_syntax_colors;
use egui::{Color32, ScrollArea, TextEdit, Ui};

/// SQL Query Editor component
#[derive(Default)]
pub struct QueryEditor {
    sql_text: String,
    is_executing: bool,
    last_execution_time: Option<std::time::Duration>,
    cursor_position: Option<egui::text::CCursor>,
    execute_requested: bool,
}

impl QueryEditor {
    pub fn new() -> Self {
        Self {
            sql_text:
                "-- Enter your SQL query here\nSELECT * FROM information_schema.tables LIMIT 10;"
                    .to_string(),
            is_executing: false,
            last_execution_time: None,
            cursor_position: None,
            execute_requested: false,
        }
    }

    pub fn render(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Toolbar
            ui.horizontal(|ui| {
                let execute_clicked = ui.button("Execute (F5)").clicked();

                if ui.button("Clear").clicked() {
                    self.sql_text.clear();
                }

                if ui.button("Format").clicked() {
                    // TODO: Implement SQL formatting
                }

                ui.separator();

                if self.is_executing {
                    ui.spinner();
                    ui.label("Executing...");
                } else if let Some(duration) = self.last_execution_time {
                    ui.label(format!("Last execution: {:.2}ms", duration.as_millis()));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Lines: {}", self.sql_text.lines().count()));
                    ui.separator();
                    ui.label(format!("Length: {}", self.sql_text.len()));
                });

                // Set execute requested flag
                if execute_clicked {
                    self.execute_requested = true;
                }
            });

            ui.separator();

            // SQL Editor
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .id_source("query_editor_scroll")
                .show(ui, |ui| {
                    let response = ui.add(
                        TextEdit::multiline(&mut self.sql_text)
                            .font(egui::TextStyle::Monospace)
                            .code_editor()
                            .desired_width(f32::INFINITY)
                            .desired_rows(15),
                    );

                    // Handle keyboard shortcuts
                    if response.has_focus() {
                        ui.input(|i| {
                            // Ctrl+A - Select all
                            if i.modifiers.ctrl && i.key_pressed(egui::Key::A) {
                                // TODO: Select all text
                            }

                            // Ctrl+/ - Toggle comment
                            if i.modifiers.ctrl && i.key_pressed(egui::Key::Slash) {
                                self.toggle_comment();
                            }

                            // Tab - Insert spaces or autocomplete
                            if i.key_pressed(egui::Key::Tab) && !i.modifiers.shift {
                                // TODO: Handle tab completion
                            }
                        });
                    }

                    // Store cursor position for potential syntax highlighting
                    // Note: cursor_range is not available in egui 0.28
                    // if let Some(cursor) = response.cursor_range {
                    //     self.cursor_position = Some(cursor.primary);
                    // }
                });

            // Status bar
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Ready");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(pos) = self.cursor_position {
                        let line = self.sql_text[..pos.index.min(self.sql_text.len())]
                            .chars()
                            .filter(|&c| c == '\n')
                            .count()
                            + 1;
                        let col = self.sql_text[..pos.index.min(self.sql_text.len())]
                            .lines()
                            .last()
                            .map(|l| l.len())
                            .unwrap_or(0)
                            + 1;
                        ui.label(format!("Line {}, Column {}", line, col));
                    }
                });
            });
        });
    }

    pub fn get_sql(&self) -> &str {
        &self.sql_text
    }

    pub fn set_sql(&mut self, sql: String) {
        self.sql_text = sql;
    }

    pub fn clear(&mut self) {
        self.sql_text.clear();
    }

    pub fn is_executing(&self) -> bool {
        self.is_executing
    }

    pub fn set_executing(&mut self, executing: bool) {
        self.is_executing = executing;
    }

    pub fn set_execution_time(&mut self, duration: std::time::Duration) {
        self.last_execution_time = Some(duration);
        self.is_executing = false;
    }

    pub fn is_execute_requested(&self) -> bool {
        self.execute_requested
    }

    pub fn clear_execute_request(&mut self) {
        self.execute_requested = false;
    }

    pub fn get_selected_text(&self) -> Option<String> {
        // TODO: Implement getting selected text
        None
    }

    pub fn insert_text(&mut self, text: &str) {
        // TODO: Insert text at cursor position
        self.sql_text.push_str(text);
    }

    fn toggle_comment(&mut self) {
        // TODO: Implement comment toggling for selected lines
        // For now, just add a comment at the beginning
        if !self.sql_text.starts_with("--") {
            self.sql_text = format!("-- {}", self.sql_text);
        } else {
            self.sql_text = self
                .sql_text
                .strip_prefix("-- ")
                .unwrap_or(&self.sql_text)
                .to_string();
        }
    }

    /// Get SQL keywords for syntax highlighting
    fn get_sql_keywords() -> Vec<&'static str> {
        vec![
            "SELECT",
            "FROM",
            "WHERE",
            "INSERT",
            "UPDATE",
            "DELETE",
            "CREATE",
            "DROP",
            "ALTER",
            "TABLE",
            "INDEX",
            "VIEW",
            "DATABASE",
            "SCHEMA",
            "FUNCTION",
            "PROCEDURE",
            "TRIGGER",
            "CONSTRAINT",
            "PRIMARY",
            "FOREIGN",
            "KEY",
            "REFERENCES",
            "UNIQUE",
            "NOT",
            "NULL",
            "DEFAULT",
            "CHECK",
            "AND",
            "OR",
            "IN",
            "EXISTS",
            "BETWEEN",
            "LIKE",
            "IS",
            "AS",
            "JOIN",
            "INNER",
            "LEFT",
            "RIGHT",
            "FULL",
            "OUTER",
            "ON",
            "GROUP",
            "BY",
            "HAVING",
            "ORDER",
            "ASC",
            "DESC",
            "LIMIT",
            "OFFSET",
            "UNION",
            "INTERSECT",
            "EXCEPT",
            "CASE",
            "WHEN",
            "THEN",
            "ELSE",
            "END",
            "IF",
            "WHILE",
            "FOR",
            "LOOP",
            "BEGIN",
            "COMMIT",
            "ROLLBACK",
            "TRANSACTION",
            "SAVEPOINT",
            "GRANT",
            "REVOKE",
            "PRIVILEGES",
            "ROLE",
            "USER",
            "PASSWORD",
            "LOGIN",
        ]
    }
}
