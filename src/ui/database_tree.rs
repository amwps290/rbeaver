use crate::database::{Column, ConnectionParams, Schema, Table};
use egui::{CollapsingHeader, ScrollArea, Ui};
use std::collections::HashMap;

/// Connection node containing all data for a specific database connection
#[derive(Debug, Clone)]
pub struct ConnectionNode {
    pub connection_id: String,
    pub connection_name: String,
    pub schemas: Vec<Schema>,
    pub tables: HashMap<String, Vec<Table>>,
    pub columns: HashMap<String, Vec<Column>>,
    pub is_connected: bool,
}

impl ConnectionNode {
    pub fn new(connection_id: String, connection_name: String) -> Self {
        Self {
            connection_id,
            connection_name,
            schemas: Vec::new(),
            tables: HashMap::new(),
            columns: HashMap::new(),
            is_connected: false,
        }
    }
}

/// Database explorer tree component
pub struct DatabaseTree {
    connections: HashMap<String, ConnectionNode>,
    saved_connections: Vec<ConnectionParams>,
    expanded_connections: HashMap<String, bool>,
    expanded_saved_connections: HashMap<String, bool>,
    expanded_schemas: HashMap<String, bool>,
    expanded_tables: HashMap<String, bool>,
    selected_item: Option<TreeItem>,

    is_loading: bool,
    schemas_needing_tables: Vec<(String, String)>, // (connection_id, schema_name)
    tables_needing_columns: Vec<(String, String, String)>, // (connection_id, schema_name, table_name)
    pending_action: Option<(ConnectionAction, String)>,    // (action, connection_id)
}

#[derive(Debug, Clone, PartialEq)]
pub enum TreeItem {
    SavedConnection(String), // Connection ID for saved connections
    Connection(String),      // Connection ID for active connections
    Schema {
        connection_id: String,
        schema: String,
    },
    Table {
        connection_id: String,
        schema: String,
        table: String,
    },
    Column {
        connection_id: String,
        schema: String,
        table: String,
        column: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionAction {
    Connect,
    Edit,
    Duplicate,
    Delete,
    CopyUrl,
}

impl DatabaseTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn render(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Toolbar
            ui.horizontal(|ui| {
                if ui.button("Refresh").clicked() {
                    // TODO: Trigger refresh
                    self.is_loading = true;
                }

                if ui.button("Expand All").clicked() {
                    self.expand_all();
                }

                if ui.button("Collapse All").clicked() {
                    self.collapse_all();
                }
            });

            ui.separator();

            if self.is_loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading database structure...");
                });
            } else if self.connections.is_empty() && self.saved_connections.is_empty() {
                ui.label("No database connections");
                ui.label("Use File > New Connection to add a connection");
            } else {
                // Tree content
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .id_source("database_tree_scroll")
                    .show(ui, |ui| {
                        self.render_tree_content(ui);
                    });
            }
        });
    }

    fn render_tree_content(&mut self, ui: &mut Ui) {
        // Render saved connections first
        for saved_connection in &self.saved_connections.clone() {
            self.render_saved_connection_node(ui, saved_connection);
        }

        // Then render active connections that are not in saved connections
        for (connection_id, connection) in &self.connections.clone() {
            // Only show if not already shown as saved connection
            if !self
                .saved_connections
                .iter()
                .any(|sc| sc.id == *connection_id)
            {
                self.render_connection_node(ui, connection_id, connection);
            }
        }
    }

    fn render_saved_connection_node(&mut self, ui: &mut Ui, saved_connection: &ConnectionParams) {
        let is_connected = self.connections.contains_key(&saved_connection.id);
        let connection_expanded = self
            .expanded_saved_connections
            .get(&saved_connection.id)
            .copied()
            .unwrap_or(false);

        let connection_icon = if is_connected { "🔗" } else { "❌" };
        let connection_label = format!("{} {}", connection_icon, saved_connection.name);

        let header_response = CollapsingHeader::new(connection_label)
            .id_source(format!("saved_connection_{}", saved_connection.id))
            .default_open(connection_expanded)
            .show(ui, |ui| {
                if is_connected {
                    // Show schemas if connected
                    if let Some(connection) = self.connections.get(&saved_connection.id) {
                        if connection.schemas.is_empty() {
                            ui.label("No schemas found");
                        } else {
                            let schemas = connection.schemas.clone();
                            for schema in &schemas {
                                self.render_schema_node(ui, &saved_connection.id, schema);
                            }
                        }
                    }
                } else {
                    ui.label("Not connected");
                    if ui.button("Connect").clicked() {
                        self.pending_action =
                            Some((ConnectionAction::Connect, saved_connection.id.clone()));
                    }
                }
            });

        // Handle expansion state
        if header_response.header_response.clicked() {
            let new_state = !connection_expanded;
            self.expanded_saved_connections
                .insert(saved_connection.id.clone(), new_state);
        }

        // Handle right-click context menu
        header_response.header_response.context_menu(|ui| {
            if ui.button("🔗 Connect").clicked() {
                self.pending_action =
                    Some((ConnectionAction::Connect, saved_connection.id.clone()));
                ui.close_menu();
            }

            if ui.button("✏️ Edit").clicked() {
                self.pending_action = Some((ConnectionAction::Edit, saved_connection.id.clone()));
                ui.close_menu();
            }

            if ui.button("📋 Duplicate").clicked() {
                self.pending_action =
                    Some((ConnectionAction::Duplicate, saved_connection.id.clone()));
                ui.close_menu();
            }

            if ui.button("📄 Copy Connection URL").clicked() {
                self.pending_action =
                    Some((ConnectionAction::CopyUrl, saved_connection.id.clone()));
                ui.close_menu();
            }

            ui.separator();

            if ui.button("🗑️ Delete").clicked() {
                self.pending_action = Some((ConnectionAction::Delete, saved_connection.id.clone()));
                ui.close_menu();
            }
        });

        // Handle double-click to connect
        if header_response.header_response.double_clicked() && !is_connected {
            self.pending_action = Some((ConnectionAction::Connect, saved_connection.id.clone()));
        }

        // Handle selection
        if header_response.header_response.clicked() {
            self.selected_item = Some(TreeItem::SavedConnection(saved_connection.id.clone()));
        }
    }

    fn render_connection_node(
        &mut self,
        ui: &mut Ui,
        connection_id: &str,
        connection: &ConnectionNode,
    ) {
        let connection_expanded = self
            .expanded_connections
            .get(connection_id)
            .copied()
            .unwrap_or(false);

        let connection_icon = if connection.is_connected {
            "🔗"
        } else {
            "❌"
        };
        let connection_label = format!("{} {}", connection_icon, connection.connection_name);

        let header_response = CollapsingHeader::new(connection_label)
            .id_source(format!("connection_{}", connection_id))
            .default_open(connection_expanded)
            .show(ui, |ui| {
                if connection.schemas.is_empty() {
                    ui.label("No schemas found");
                } else {
                    for schema in &connection.schemas {
                        self.render_schema_node(ui, connection_id, schema);
                    }
                }
            });

        // Track expansion state
        if header_response.header_response.clicked() {
            let new_state = !connection_expanded;
            self.expanded_connections
                .insert(connection_id.to_string(), new_state);
        }

        // Handle connection selection
        if header_response.header_response.secondary_clicked() {
            self.selected_item = Some(TreeItem::Connection(connection_id.to_string()));
            // TODO: Show connection context menu
        }
    }

    fn render_schema_node(&mut self, ui: &mut Ui, connection_id: &str, schema: &Schema) {
        let schema_key = format!("{}.{}", connection_id, schema.name);
        let schema_expanded = self
            .expanded_schemas
            .get(&schema_key)
            .copied()
            .unwrap_or(false);

        let header_response = CollapsingHeader::new(format!("📁 {}", schema.name))
            .id_source(format!("schema_{}_{}", connection_id, schema.name))
            .default_open(schema_expanded)
            .show(ui, |ui| {
                // Show tables in this schema
                if let Some(connection) = self.connections.get(connection_id) {
                    if let Some(tables) = connection.tables.get(&schema.name).cloned() {
                        for table in &tables {
                            self.render_table_node(ui, connection_id, &schema.name, table);
                        }
                    } else {
                        ui.label("Loading tables...");
                    }
                }
            });

        // Track expansion state
        if header_response.header_response.clicked() {
            let new_state = !schema_expanded;
            self.expanded_schemas.insert(schema_key, new_state);

            // Load tables if expanding for the first time
            if let Some(connection) = self.connections.get(connection_id) {
                if new_state && !connection.tables.contains_key(&schema.name) {
                    self.schemas_needing_tables
                        .push((connection_id.to_string(), schema.name.clone()));
                }
            }
        }

        // Handle schema selection
        if header_response.header_response.secondary_clicked() {
            self.selected_item = Some(TreeItem::Schema {
                connection_id: connection_id.to_string(),
                schema: schema.name.clone(),
            });
            // TODO: Show schema context menu
        }
    }

    fn render_table_node(
        &mut self,
        ui: &mut Ui,
        connection_id: &str,
        schema_name: &str,
        table: &Table,
    ) {
        let table_key = format!("{}.{}.{}", connection_id, schema_name, table.name);
        let table_expanded = self
            .expanded_tables
            .get(&table_key)
            .copied()
            .unwrap_or(false);

        let table_icon = match table.table_type.as_str() {
            "VIEW" => "👁",
            "MATERIALIZED VIEW" => "📊",
            _ => "📋",
        };

        let header_response = CollapsingHeader::new(format!("{} {}", table_icon, table.name))
            .id_source(format!(
                "table_{}_{}_{}",
                connection_id, schema_name, table.name
            ))
            .default_open(table_expanded)
            .show(ui, |ui| {
                // Show columns in this table
                if let Some(connection) = self.connections.get(connection_id) {
                    let column_key = format!("{}.{}", schema_name, table.name);
                    if let Some(columns) = connection.columns.get(&column_key).cloned() {
                        for column in &columns {
                            self.render_column_node(
                                ui,
                                connection_id,
                                schema_name,
                                &table.name,
                                column,
                            );
                        }
                    } else {
                        ui.label("Loading columns...");
                    }
                }
            });

        // Track expansion state
        if header_response.header_response.clicked() {
            let new_state = !table_expanded;
            self.expanded_tables.insert(table_key.clone(), new_state);

            // Load columns if expanding for the first time
            if let Some(connection) = self.connections.get(connection_id) {
                let column_key = format!("{}.{}", schema_name, table.name);
                if new_state && !connection.columns.contains_key(&column_key) {
                    self.tables_needing_columns.push((
                        connection_id.to_string(),
                        schema_name.to_string(),
                        table.name.clone(),
                    ));
                }
            }
        }

        // Handle table selection
        if header_response.header_response.secondary_clicked() {
            self.selected_item = Some(TreeItem::Table {
                connection_id: connection_id.to_string(),
                schema: schema_name.to_string(),
                table: table.name.clone(),
            });
            // TODO: Show table context menu
        }

        // Handle double-click to show table data
        if header_response.header_response.double_clicked() {
            // TODO: Trigger "SELECT * FROM table" query
        }
    }

    fn render_column_node(
        &mut self,
        ui: &mut Ui,
        connection_id: &str,
        schema_name: &str,
        table_name: &str,
        column: &Column,
    ) {
        let column_icon = if column.is_primary_key {
            "🔑"
        } else if !column.is_nullable {
            "🔒"
        } else {
            "📄"
        };

        let column_text = format!("{} {} ({})", column_icon, column.name, column.data_type);

        let response = ui.selectable_label(
            self.selected_item
                == Some(TreeItem::Column {
                    connection_id: connection_id.to_string(),
                    schema: schema_name.to_string(),
                    table: table_name.to_string(),
                    column: column.name.clone(),
                }),
            column_text,
        );

        if response.clicked() {
            self.selected_item = Some(TreeItem::Column {
                connection_id: connection_id.to_string(),
                schema: schema_name.to_string(),
                table: table_name.to_string(),
                column: column.name.clone(),
            });
        }

        // Show column details on hover
        let mut hover_text = format!(
            "Type: {}\nNullable: {}",
            column.data_type, column.is_nullable
        );
        if let Some(default) = &column.default_value {
            hover_text.push_str(&format!("\nDefault: {}", default));
        }
        if let Some(comment) = &column.comment {
            hover_text.push_str(&format!("\nComment: {}", comment));
        }
        let response = response.on_hover_text(hover_text);

        // Handle right-click for context menu
        if response.secondary_clicked() {
            // TODO: Show column context menu
        }
    }

    pub fn add_connection(&mut self, connection_id: String, connection_name: String) {
        let connection = ConnectionNode::new(connection_id.clone(), connection_name);
        self.connections.insert(connection_id, connection);
    }

    pub fn remove_connection(&mut self, connection_id: &str) {
        self.connections.remove(connection_id);
        self.expanded_connections.remove(connection_id);
        // Clean up related expansion states
        self.expanded_schemas
            .retain(|key, _| !key.starts_with(&format!("{}.", connection_id)));
        self.expanded_tables
            .retain(|key, _| !key.starts_with(&format!("{}.", connection_id)));
    }

    pub fn set_connection_status(&mut self, connection_id: &str, is_connected: bool) {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            connection.is_connected = is_connected;
        }
    }

    pub fn set_schemas(&mut self, connection_id: &str, schemas: Vec<Schema>) {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            connection.schemas = schemas;
        }
        self.is_loading = false;
    }

    pub fn set_tables(&mut self, connection_id: &str, schema: String, tables: Vec<Table>) {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            connection.tables.insert(schema, tables);
        }
    }

    pub fn set_columns(
        &mut self,
        connection_id: &str,
        schema: String,
        table: String,
        columns: Vec<Column>,
    ) {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            let key = format!("{}.{}", schema, table);
            connection.columns.insert(key, columns);
        }
    }

    pub fn clear(&mut self) {
        self.connections.clear();
        self.expanded_connections.clear();
        self.expanded_schemas.clear();
        self.expanded_tables.clear();
        self.selected_item = None;
        self.is_loading = false;
    }

    pub fn get_selected_item(&self) -> Option<&TreeItem> {
        self.selected_item.as_ref()
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    fn expand_all(&mut self) {
        // Expand all connections
        for connection_id in self.connections.keys() {
            self.expanded_connections
                .insert(connection_id.clone(), true);
        }

        // Expand all schemas
        for (connection_id, connection) in &self.connections {
            for schema in &connection.schemas {
                let schema_key = format!("{}.{}", connection_id, schema.name);
                self.expanded_schemas.insert(schema_key, true);
            }
        }

        // Expand all tables
        for (connection_id, connection) in &self.connections {
            for (schema_name, tables) in &connection.tables {
                for table in tables {
                    let table_key = format!("{}.{}.{}", connection_id, schema_name, table.name);
                    self.expanded_tables.insert(table_key, true);
                }
            }
        }
    }

    fn collapse_all(&mut self) {
        self.expanded_connections.clear();
        self.expanded_schemas.clear();
        self.expanded_tables.clear();
    }

    /// Generate SQL for selected item
    pub fn get_sql_for_selected(&self) -> Option<String> {
        match &self.selected_item {
            Some(TreeItem::Table { schema, table, .. }) => Some(format!(
                "SELECT * FROM \"{}\".\"{}\" LIMIT 100;",
                schema, table
            )),
            Some(TreeItem::Column {
                schema,
                table,
                column,
                ..
            }) => Some(format!(
                "SELECT \"{}\" FROM \"{}\".\"{}\" LIMIT 100;",
                column, schema, table
            )),
            _ => None,
        }
    }

    pub fn get_schemas_needing_tables(&mut self) -> Vec<(String, String)> {
        let schemas = self.schemas_needing_tables.clone();
        self.schemas_needing_tables.clear();
        schemas
    }

    pub fn get_tables_needing_columns(&mut self) -> Vec<(String, String, String)> {
        let tables = self.tables_needing_columns.clone();
        self.tables_needing_columns.clear();
        tables
    }

    /// Set saved connections
    pub fn set_saved_connections(&mut self, connections: Vec<ConnectionParams>) {
        self.saved_connections = connections;
    }

    /// Get pending action and clear it
    pub fn get_pending_action(&mut self) -> Option<(ConnectionAction, String)> {
        self.pending_action.take()
    }

    /// Update saved connection status
    pub fn update_saved_connection_status(&mut self, connection_id: &str, is_connected: bool) {
        // Update the connection status in the tree
        if is_connected {
            // Connection is now active, ensure it's in the connections map
            if !self.connections.contains_key(connection_id) {
                // This will be handled by the main app
            }
        } else {
            // Connection is no longer active, but keep it in saved connections
        }
    }

    /// Refresh saved connections from settings
    pub fn refresh_saved_connections(&mut self, connections: Vec<ConnectionParams>) {
        self.saved_connections = connections;

        // Clean up expansion states for removed connections
        let connection_ids: std::collections::HashSet<String> = self
            .saved_connections
            .iter()
            .map(|c| c.id.clone())
            .collect();

        self.expanded_saved_connections
            .retain(|id, _| connection_ids.contains(id));
    }
}

impl Default for DatabaseTree {
    fn default() -> Self {
        Self {
            connections: HashMap::new(),
            saved_connections: Vec::new(),
            expanded_connections: HashMap::new(),
            expanded_saved_connections: HashMap::new(),
            expanded_schemas: HashMap::new(),
            expanded_tables: HashMap::new(),
            selected_item: None,

            is_loading: false,
            schemas_needing_tables: Vec::new(),
            tables_needing_columns: Vec::new(),
            pending_action: None,
        }
    }
}
