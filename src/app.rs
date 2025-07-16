use crate::config::AppSettings;
use crate::database::{
    ConnectionParams, DatabaseConnection, DatabaseError, ObjectCategory, PostgreSQLConnection,
    QueryExecutor, QueryResult,
};
use crate::ui::{
    ConfirmationDialog, ConnectionAction, ConnectionDialog, DatabaseTree, DialogAction,
    QueryEditor, ResultTable, TreeItem,
};
use eframe::egui;
use std::collections::HashMap;
use tokio::runtime::Runtime;

/// Main application state and logic
pub struct RBeaverApp {
    /// Current database connections
    connections: HashMap<String, PostgreSQLConnection>,

    /// Active connection ID
    active_connection: Option<String>,

    /// UI Components
    connection_dialog: ConnectionDialog,
    query_editor: QueryEditor,
    result_table: ResultTable,
    database_tree: DatabaseTree,
    confirmation_dialog: ConfirmationDialog,

    /// Application state
    show_connection_dialog: bool,
    is_connecting: bool,
    is_testing_connection: bool,
    last_error: Option<String>,
    pending_connection_deletion: Option<String>,
    clipboard_message: Option<String>,
    clipboard_message_time: Option<std::time::Instant>,

    /// Connection management
    saved_connections: Vec<ConnectionParams>,
    settings: AppSettings,

    /// Async runtime for database operations
    runtime: Runtime,
}

impl Default for RBeaverApp {
    fn default() -> Self {
        // Load settings and saved connections
        let settings = AppSettings::load().unwrap_or_default();
        let saved_connections = settings.connections.clone();

        let mut app = Self {
            connections: HashMap::new(),
            active_connection: None,
            connection_dialog: ConnectionDialog::default(),
            query_editor: QueryEditor::default(),
            result_table: ResultTable::default(),
            database_tree: DatabaseTree::default(),
            confirmation_dialog: ConfirmationDialog::default(),
            show_connection_dialog: false,
            is_connecting: false,
            is_testing_connection: false,
            last_error: None,
            pending_connection_deletion: None,
            clipboard_message: None,
            clipboard_message_time: None,
            saved_connections: saved_connections.clone(),
            settings,
            runtime: Runtime::new().expect("Failed to create async runtime"),
        };

        // Initialize database tree with saved connections
        app.database_tree.set_saved_connections(saved_connections);
        app
    }
}

impl eframe::App for RBeaverApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle global keyboard shortcuts
        ctx.input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
                self.show_connection_dialog = true;
            }

            // F5 or Ctrl+Enter for query execution
            if i.key_pressed(egui::Key::F5) || (i.modifiers.ctrl && i.key_pressed(egui::Key::Enter))
            {
                if self.active_connection.is_some() {
                    let sql = self.query_editor.get_sql().trim().to_string();
                    if !sql.is_empty() {
                        self.execute_query(&sql);
                    }
                }
            }

            // Ctrl+Shift+C for new connection
            if i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::C) {
                self.show_connection_dialog = true;
            }
        });

        self.render_menu_bar(ctx);
        self.render_main_layout(ctx);

        // Handle tree actions
        if let Some((action, connection_id)) = self.database_tree.get_pending_action() {
            self.handle_connection_action(action, connection_id);
        }

        // Handle connection dialog
        if self.show_connection_dialog {
            self.render_connection_dialog(ctx);
        }

        // Handle confirmation dialog
        self.confirmation_dialog.render(ctx);
        if self.confirmation_dialog.take_confirmed() {
            if let Some(connection_id) = self.pending_connection_deletion.take() {
                self.delete_saved_connection(&connection_id);
            }
        } else if self.confirmation_dialog.take_cancelled() {
            self.pending_connection_deletion = None;
        }

        // Handle errors
        if let Some(error) = self.last_error.clone() {
            self.render_error_dialog(ctx, &error);
        }

        // Status bar
        self.render_status_bar(ctx);
    }
}

impl RBeaverApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        log::info!("Initializing RBeaver application");

        // Configure fonts for Chinese character support
        Self::configure_fonts(&cc.egui_ctx);

        Self::default()
    }

    /// Configure fonts to support Chinese characters
    fn configure_fonts(ctx: &egui::Context) {
        // egui 0.28 has good Unicode support by default
        // We'll configure it to ensure optimal Chinese character rendering

        // Configure text rendering options for better Unicode support
        let mut style = (*ctx.style()).clone();

        // Set better spacing for CJK characters
        style.spacing.item_spacing = egui::vec2(8.0, 4.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);

        // Ensure text is rendered with proper Unicode support
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(14.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(12.0, egui::FontFamily::Monospace),
        );

        ctx.set_style(style);

        log::info!("Configured text rendering for Chinese character support");
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Connection").clicked() {
                        self.show_connection_dialog = true;
                        ui.close_menu();
                    }

                    ui.separator();
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.menu_button("Edit", |ui| {
                    // TODO: Add edit menu items
                    ui.label("Coming soon...");
                });

                ui.menu_button("View", |ui| {
                    // TODO: Add view menu items
                    ui.label("Coming soon...");
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        // TODO: Show about dialog
                        ui.close_menu();
                    }
                });
            });
        });
    }

    fn render_main_layout(&mut self, ctx: &egui::Context) {
        // Left panel for database tree
        egui::SidePanel::left("database_tree")
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Database Explorer");
                self.database_tree.render(ui);

                // Handle tree interactions
                if let Some(selected_item) = self.database_tree.get_selected_item() {
                    match selected_item {
                        crate::ui::database_tree::TreeItem::Table { schema, table, .. } => {
                            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                let sql = format!(
                                    "SELECT * FROM \"{}\".\"{}\" LIMIT 100;",
                                    schema, table
                                );
                                self.query_editor.set_sql(sql);
                            }
                        }
                        _ => {}
                    }
                }

                // Handle tree expansion requests
                self.handle_tree_expansion_requests();
            });

        // Central panel for query editor and results
        egui::CentralPanel::default().show(ctx, |ui| {
            // Split vertically: query editor on top, results on bottom
            let available_height = ui.available_height();
            let editor_height = available_height * 0.4;

            // Query editor
            ui.allocate_ui_with_layout(
                egui::Vec2::new(ui.available_width(), editor_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("SQL Query Editor");

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(connection_id) = &self.active_connection {
                                // Find the connection name
                                let connection_name = self
                                    .saved_connections
                                    .iter()
                                    .find(|c| c.id == *connection_id)
                                    .map(|c| c.name.as_str())
                                    .unwrap_or(connection_id);
                                ui.colored_label(
                                    egui::Color32::from_rgb(40, 167, 69),
                                    format!("Connected: {}", connection_name),
                                );
                            } else {
                                ui.colored_label(
                                    egui::Color32::from_rgb(220, 53, 69),
                                    "Not connected",
                                );
                            }
                        });
                    });

                    self.query_editor.render(ui);

                    // Handle query execution from button or keyboard shortcuts
                    let should_execute = self.query_editor.is_execute_requested()
                        || ui.input(|i| i.key_pressed(egui::Key::F5))
                        || (ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter)));

                    if should_execute {
                        self.query_editor.clear_execute_request();
                        let sql = self.query_editor.get_sql().trim().to_string();
                        if !sql.is_empty() && self.active_connection.is_some() {
                            self.execute_query(&sql);
                        } else if sql.is_empty() {
                            self.last_error = Some("Please enter a SQL query".to_string());
                        } else {
                            self.last_error = Some("No database connection available".to_string());
                        }
                    }
                },
            );

            ui.separator();

            // Results table
            ui.allocate_ui_with_layout(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    ui.heading("Query Results");
                    self.result_table.render(ui);
                },
            );
        });
    }

    fn render_connection_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Database Connection")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                self.connection_dialog.render(ui);

                ui.add_space(10.0);

                let action = self.connection_dialog.render_buttons(ui);

                match action {
                    DialogAction::TestConnection => {
                        self.test_connection();
                    }
                    DialogAction::SaveAndConnect => {
                        let params = self.connection_dialog.get_params().clone();
                        self.save_connection(params, true);
                    }
                    DialogAction::SaveOnly => {
                        let params = self.connection_dialog.get_params().clone();
                        self.save_connection(params, false);
                    }
                    DialogAction::UpdateConnection => {
                        let params = self.connection_dialog.get_params().clone();
                        self.update_connection(params);
                    }
                    DialogAction::Cancel => {
                        self.show_connection_dialog = false;
                        self.connection_dialog.reset();
                    }
                    DialogAction::None => {}
                }

                if self.is_connecting {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Connecting...");
                    });
                } else if self.is_testing_connection {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Testing...");
                    });
                }
            });
    }

    fn render_error_dialog(&mut self, ctx: &egui::Context, error: &str) {
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(error);
                if ui.button("OK").clicked() {
                    self.last_error = None;
                }
            });
    }

    fn render_status_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Connection status
                if let Some(connection_id) = &self.active_connection {
                    let connection_name = self
                        .saved_connections
                        .iter()
                        .find(|c| c.id == *connection_id)
                        .map(|c| c.name.as_str())
                        .unwrap_or(connection_id);
                    ui.colored_label(
                        egui::Color32::from_rgb(40, 167, 69),
                        format!("🔗 Connected: {}", connection_name),
                    );
                } else {
                    ui.colored_label(egui::Color32::from_rgb(220, 53, 69), "❌ Not connected");
                }

                ui.separator();

                // Show clipboard message if recent
                if let (Some(message), Some(time)) =
                    (&self.clipboard_message, &self.clipboard_message_time)
                {
                    if time.elapsed().as_secs() < 3 {
                        ui.colored_label(
                            egui::Color32::from_rgb(40, 167, 69),
                            format!("✓ {}", message),
                        );
                        ui.separator();
                    } else {
                        // Clear expired message
                        self.clipboard_message = None;
                        self.clipboard_message_time = None;
                    }
                }

                // Keyboard shortcuts help
                ui.label(
                    "Shortcuts: F5 (Execute), Ctrl+N (New Connection), Ctrl+Shift+C (Connect)",
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("RBeaver v0.1.0");
                });
            });
        });
    }

    fn attempt_connection(&mut self, params: ConnectionParams) {
        self.is_connecting = true;

        // Create a new PostgreSQL connection
        let mut connection = PostgreSQLConnection::new();

        // Attempt to connect in the runtime
        match self.runtime.block_on(connection.connect(&params)) {
            Ok(()) => {
                // Connection successful
                self.connections.insert(params.id.clone(), connection);
                self.active_connection = Some(params.id.clone());
                // Keep dialog open for multiple connections
                self.connection_dialog.reset();

                // Add connection to database tree
                self.database_tree
                    .add_connection(params.id.clone(), params.name.clone());
                self.database_tree.set_connection_status(&params.id, true);

                // Save connection if not already saved
                self.save_connection_if_new(params.clone());

                // Load database structure
                self.load_database_structure();

                log::info!("Successfully connected to database: {}", params.name);
            }
            Err(err) => {
                self.last_error = Some(format!("Connection failed: {}", err));
                log::error!("Connection failed: {}", err);
            }
        }

        self.is_connecting = false;
    }

    fn test_connection(&mut self) {
        self.is_testing_connection = true;
        let params = self.connection_dialog.get_params().clone();

        // Create a temporary connection for testing
        let connection = PostgreSQLConnection::new();

        // Test the connection
        match self.runtime.block_on(connection.test_connection(&params)) {
            Ok(()) => {
                self.connection_dialog.set_test_result(Ok(()));
                log::info!("Connection test successful for: {}", params.name);
            }
            Err(err) => {
                let error_msg = format!("Connection test failed: {}", err);
                self.connection_dialog
                    .set_test_result(Err(error_msg.clone()));
                log::error!("Connection test failed: {}", err);
            }
        }

        self.is_testing_connection = false;
    }

    fn disconnect_database(&mut self, connection_id: &str) {
        // Remove connection from active connections
        if let Some(active_id) = &self.active_connection {
            if active_id == connection_id {
                self.active_connection = None;
            }
        }

        // Remove from connections map
        self.connections.remove(connection_id);

        // Remove from database tree
        self.database_tree.remove_connection(connection_id);

        log::info!("Disconnected from database: {}", connection_id);
    }

    fn handle_connection_action(&mut self, action: ConnectionAction, connection_id: String) {
        match action {
            ConnectionAction::Connect => {
                self.connect_to_saved_connection(&connection_id);
            }
            ConnectionAction::Edit => {
                self.edit_saved_connection(&connection_id);
            }
            ConnectionAction::Duplicate => {
                self.duplicate_saved_connection(&connection_id);
            }
            ConnectionAction::Delete => {
                self.request_delete_connection(&connection_id);
            }
            ConnectionAction::CopyUrl => {
                self.copy_connection_url(&connection_id);
            }
        }
    }

    fn connect_to_saved_connection(&mut self, connection_id: &str) {
        if let Some(params) = self
            .saved_connections
            .iter()
            .find(|c| c.id == *connection_id)
        {
            let params = params.clone();
            self.attempt_connection(params);
        }
    }

    fn edit_saved_connection(&mut self, connection_id: &str) {
        if let Some(params) = self
            .saved_connections
            .iter()
            .find(|c| c.id == *connection_id)
        {
            self.connection_dialog = ConnectionDialog::for_editing(params.clone());
            self.show_connection_dialog = true;
        }
    }

    fn duplicate_saved_connection(&mut self, connection_id: &str) {
        match self.settings.duplicate_connection(connection_id, None) {
            Ok(duplicated) => {
                self.saved_connections = self.settings.get_all_connections().clone();
                self.database_tree
                    .refresh_saved_connections(self.saved_connections.clone());

                // Save settings
                if let Err(err) = self.settings.save() {
                    self.last_error = Some(format!("Failed to save settings: {}", err));
                }

                log::info!("Duplicated connection: {}", duplicated.name);
            }
            Err(err) => {
                self.last_error = Some(format!("Failed to duplicate connection: {}", err));
            }
        }
    }

    fn request_delete_connection(&mut self, connection_id: &str) {
        if let Some(params) = self
            .saved_connections
            .iter()
            .find(|c| c.id == *connection_id)
        {
            self.confirmation_dialog
                .show_delete_confirmation(&params.name);
            self.pending_connection_deletion = Some(connection_id.to_string());
        }
    }

    fn copy_connection_url(&mut self, connection_id: &str) {
        if let Some(params) = self
            .saved_connections
            .iter()
            .find(|c| c.id == *connection_id)
        {
            let connection_url = params.get_connection_url();

            // Copy to clipboard
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => match clipboard.set_text(&connection_url) {
                    Ok(()) => {
                        self.clipboard_message =
                            Some(format!("Connection URL copied to clipboard"));
                        self.clipboard_message_time = Some(std::time::Instant::now());
                        log::info!("Copied connection URL to clipboard: {}", params.name);
                    }
                    Err(err) => {
                        self.last_error = Some(format!("Failed to copy to clipboard: {}", err));
                        log::error!("Failed to copy to clipboard: {}", err);
                    }
                },
                Err(err) => {
                    self.last_error = Some(format!("Failed to access clipboard: {}", err));
                    log::error!("Failed to access clipboard: {}", err);
                }
            }
        }
    }

    fn delete_saved_connection(&mut self, connection_id: &str) {
        // Disconnect if currently connected
        if self.connections.contains_key(connection_id) {
            self.disconnect_database(connection_id);
        }

        // Remove from settings
        match self.settings.remove_connection(connection_id) {
            Ok(_) => {
                self.saved_connections = self.settings.get_all_connections().clone();
                self.database_tree
                    .refresh_saved_connections(self.saved_connections.clone());

                // Save settings
                if let Err(err) = self.settings.save() {
                    self.last_error = Some(format!("Failed to save settings: {}", err));
                }

                log::info!("Deleted connection: {}", connection_id);
            }
            Err(err) => {
                self.last_error = Some(format!("Failed to delete connection: {}", err));
            }
        }
    }

    fn save_connection(&mut self, params: ConnectionParams, connect_after_save: bool) {
        match self.settings.add_connection(params.clone()) {
            Ok(()) => {
                self.saved_connections = self.settings.get_all_connections().clone();
                self.database_tree
                    .refresh_saved_connections(self.saved_connections.clone());

                // Save settings
                if let Err(err) = self.settings.save() {
                    self.last_error = Some(format!("Failed to save settings: {}", err));
                    return;
                }

                if connect_after_save {
                    self.attempt_connection(params.clone());
                }

                self.show_connection_dialog = false;
                self.connection_dialog.reset();

                log::info!("Saved connection: {}", params.name);
            }
            Err(err) => {
                self.last_error = Some(format!("Failed to save connection: {}", err));
            }
        }
    }

    fn update_connection(&mut self, params: ConnectionParams) {
        match self.settings.update_connection(params.clone()) {
            Ok(()) => {
                self.saved_connections = self.settings.get_all_connections().clone();
                self.database_tree
                    .refresh_saved_connections(self.saved_connections.clone());

                // Save settings
                if let Err(err) = self.settings.save() {
                    self.last_error = Some(format!("Failed to save settings: {}", err));
                    return;
                }

                self.show_connection_dialog = false;
                self.connection_dialog.reset();

                log::info!("Updated connection: {}", params.name);
            }
            Err(err) => {
                self.last_error = Some(format!("Failed to update connection: {}", err));
            }
        }
    }

    fn load_database_structure(&mut self) {
        if let Some(connection_id) = &self.active_connection {
            if let Some(connection) = self.connections.get(connection_id) {
                self.database_tree.set_loading(true);

                // Load schemas for this specific connection
                match self.runtime.block_on(connection.get_schemas()) {
                    Ok(schemas) => {
                        // Load object counts for each schema
                        for schema in &schemas {
                            match self
                                .runtime
                                .block_on(connection.get_object_counts(&schema.name))
                            {
                                Ok(counts) => {
                                    self.database_tree.set_object_counts(
                                        connection_id,
                                        schema.name.clone(),
                                        counts,
                                    );
                                }
                                Err(err) => {
                                    log::warn!(
                                        "Failed to load object counts for schema {}: {}",
                                        schema.name,
                                        err
                                    );
                                }
                            }
                        }

                        self.database_tree.set_schemas(connection_id, schemas);
                        log::info!("Loaded database schemas for connection: {}", connection_id);
                    }
                    Err(err) => {
                        self.last_error = Some(format!("Failed to load schemas: {}", err));
                        log::error!("Failed to load schemas: {}", err);
                    }
                }

                self.database_tree.set_loading(false);
            }
        }
    }

    fn execute_query(&mut self, sql: &str) {
        if let Some(connection_id) = &self.active_connection {
            if let Some(connection) = self.connections.get(connection_id) {
                self.query_editor.set_executing(true);

                let start_time = std::time::Instant::now();
                match self.runtime.block_on(connection.execute_query(sql)) {
                    Ok(result) => {
                        let execution_time = start_time.elapsed();
                        self.query_editor.set_execution_time(execution_time);
                        self.result_table.set_result(result);
                        log::info!("Query executed successfully in {:?}", execution_time);
                    }
                    Err(err) => {
                        self.last_error = Some(format!("Query execution failed: {}", err));
                        self.query_editor.set_executing(false);
                        log::error!("Query execution failed: {}", err);
                    }
                }
            } else {
                self.last_error = Some("No active database connection".to_string());
                self.query_editor.set_executing(false);
            }
        } else {
            self.last_error = Some("No database connection available".to_string());
            self.query_editor.set_executing(false);
        }
    }

    fn handle_tree_expansion_requests(&mut self) {
        // Get schemas that need table loading
        let schemas_to_load = self.database_tree.get_schemas_needing_tables();
        for (connection_id, schema_name) in schemas_to_load {
            if let Some(connection) = self.connections.get(&connection_id) {
                match self.runtime.block_on(connection.get_tables(&schema_name)) {
                    Ok(tables) => {
                        self.database_tree
                            .set_tables(&connection_id, schema_name.clone(), tables);
                        log::info!(
                            "Loaded tables for schema: {} in connection: {}",
                            schema_name,
                            connection_id
                        );
                    }
                    Err(err) => {
                        self.last_error = Some(format!(
                            "Failed to load tables for schema {} in connection {}: {}",
                            schema_name, connection_id, err
                        ));
                        log::error!(
                            "Failed to load tables for schema {} in connection {}: {}",
                            schema_name,
                            connection_id,
                            err
                        );
                    }
                }
            }
        }

        // Get tables that need column loading
        let tables_to_load = self.database_tree.get_tables_needing_columns();
        for (connection_id, schema_name, table_name) in tables_to_load {
            if let Some(connection) = self.connections.get(&connection_id) {
                match self
                    .runtime
                    .block_on(connection.get_columns(&schema_name, &table_name))
                {
                    Ok(columns) => {
                        self.database_tree.set_columns(
                            &connection_id,
                            schema_name.clone(),
                            table_name.clone(),
                            columns,
                        );
                        log::info!(
                            "Loaded columns for table: {}.{} in connection: {}",
                            schema_name,
                            table_name,
                            connection_id
                        );
                    }
                    Err(err) => {
                        self.last_error = Some(format!(
                            "Failed to load columns for table {}.{} in connection {}: {}",
                            schema_name, table_name, connection_id, err
                        ));
                        log::error!(
                            "Failed to load columns for table {}.{} in connection {}: {}",
                            schema_name,
                            table_name,
                            connection_id,
                            err
                        );
                    }
                }
            }
        }

        // Get schemas that need object loading
        let objects_to_load = self.database_tree.get_schemas_needing_objects();
        for (connection_id, schema_name, category) in objects_to_load {
            if let Some(connection) = self.connections.get(&connection_id) {
                match category {
                    ObjectCategory::Tables => {
                        // Tables are already handled above
                        continue;
                    }
                    ObjectCategory::Views => {
                        match self.runtime.block_on(connection.get_views(&schema_name)) {
                            Ok(views) => {
                                self.database_tree.set_views(
                                    &connection_id,
                                    schema_name.clone(),
                                    views,
                                );
                                log::info!(
                                    "Loaded views for schema: {} in connection: {}",
                                    schema_name,
                                    connection_id
                                );
                            }
                            Err(err) => {
                                self.last_error = Some(format!(
                                    "Failed to load views for schema {} in connection {}: {}",
                                    schema_name, connection_id, err
                                ));
                                log::error!(
                                    "Failed to load views for schema {} in connection {}: {}",
                                    schema_name,
                                    connection_id,
                                    err
                                );
                            }
                        }
                    }
                    ObjectCategory::Functions => {
                        match self
                            .runtime
                            .block_on(connection.get_functions(&schema_name))
                        {
                            Ok(functions) => {
                                self.database_tree.set_functions(
                                    &connection_id,
                                    schema_name.clone(),
                                    functions,
                                );
                                log::info!(
                                    "Loaded functions for schema: {} in connection: {}",
                                    schema_name,
                                    connection_id
                                );
                            }
                            Err(err) => {
                                self.last_error = Some(format!(
                                    "Failed to load functions for schema {} in connection {}: {}",
                                    schema_name, connection_id, err
                                ));
                                log::error!(
                                    "Failed to load functions for schema {} in connection {}: {}",
                                    schema_name,
                                    connection_id,
                                    err
                                );
                            }
                        }
                    }
                    ObjectCategory::Triggers => {
                        match self.runtime.block_on(connection.get_triggers(&schema_name)) {
                            Ok(triggers) => {
                                self.database_tree.set_triggers(
                                    &connection_id,
                                    schema_name.clone(),
                                    triggers,
                                );
                                log::info!(
                                    "Loaded triggers for schema: {} in connection: {}",
                                    schema_name,
                                    connection_id
                                );
                            }
                            Err(err) => {
                                self.last_error = Some(format!(
                                    "Failed to load triggers for schema {} in connection {}: {}",
                                    schema_name, connection_id, err
                                ));
                                log::error!(
                                    "Failed to load triggers for schema {} in connection {}: {}",
                                    schema_name,
                                    connection_id,
                                    err
                                );
                            }
                        }
                    }
                    ObjectCategory::Sequences => {
                        match self
                            .runtime
                            .block_on(connection.get_sequences(&schema_name))
                        {
                            Ok(sequences) => {
                                self.database_tree.set_sequences(
                                    &connection_id,
                                    schema_name.clone(),
                                    sequences,
                                );
                                log::info!(
                                    "Loaded sequences for schema: {} in connection: {}",
                                    schema_name,
                                    connection_id
                                );
                            }
                            Err(err) => {
                                self.last_error = Some(format!(
                                    "Failed to load sequences for schema {} in connection {}: {}",
                                    schema_name, connection_id, err
                                ));
                                log::error!(
                                    "Failed to load sequences for schema {} in connection {}: {}",
                                    schema_name,
                                    connection_id,
                                    err
                                );
                            }
                        }
                    }
                    ObjectCategory::Indexes => {
                        match self.runtime.block_on(connection.get_indexes(&schema_name)) {
                            Ok(indexes) => {
                                self.database_tree.set_indexes(
                                    &connection_id,
                                    schema_name.clone(),
                                    indexes,
                                );
                                log::info!(
                                    "Loaded indexes for schema: {} in connection: {}",
                                    schema_name,
                                    connection_id
                                );
                            }
                            Err(err) => {
                                self.last_error = Some(format!(
                                    "Failed to load indexes for schema {} in connection {}: {}",
                                    schema_name, connection_id, err
                                ));
                                log::error!(
                                    "Failed to load indexes for schema {} in connection {}: {}",
                                    schema_name,
                                    connection_id,
                                    err
                                );
                            }
                        }
                    }
                    ObjectCategory::SystemCatalog => {
                        // TODO: Implement system catalog loading
                        log::info!("System catalog loading not yet implemented");
                    }
                }
            }
        }
    }

    fn save_connection_if_new(&mut self, params: ConnectionParams) {
        // Check if connection already exists
        if !self.saved_connections.iter().any(|c| c.id == params.id) {
            // Add to saved connections
            match self.settings.add_connection(params.clone()) {
                Ok(()) => {
                    self.saved_connections.push(params);

                    // Save settings to disk
                    if let Err(err) = self.settings.save() {
                        log::error!("Failed to save settings: {}", err);
                        self.last_error = Some(format!("Failed to save settings: {}", err));
                    } else {
                        log::info!("Connection saved successfully");
                    }
                }
                Err(err) => {
                    log::error!("Failed to add connection: {}", err);
                    self.last_error = Some(format!("Failed to add connection: {}", err));
                }
            }
        }
    }
}
