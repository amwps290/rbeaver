use crate::database::{
    Column, ConnectionParams, Function, Index, ObjectCategory, ObjectCounts, Schema, Sequence,
    Table, Trigger, View,
};
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
    pub views: HashMap<String, Vec<View>>,
    pub functions: HashMap<String, Vec<Function>>,
    pub triggers: HashMap<String, Vec<Trigger>>,
    pub sequences: HashMap<String, Vec<Sequence>>,
    pub indexes: HashMap<String, Vec<Index>>,
    pub object_counts: HashMap<String, ObjectCounts>,
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
            views: HashMap::new(),
            functions: HashMap::new(),
            triggers: HashMap::new(),
            sequences: HashMap::new(),
            indexes: HashMap::new(),
            object_counts: HashMap::new(),
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
    expanded_object_categories: HashMap<String, bool>, // (connection_id.schema.category)
    expanded_tables: HashMap<String, bool>,
    selected_item: Option<TreeItem>,

    // Search and filtering
    search_text: String,
    show_search: bool,

    is_loading: bool,
    schemas_needing_tables: Vec<(String, String)>, // (connection_id, schema_name)
    schemas_needing_objects: Vec<(String, String, ObjectCategory)>, // (connection_id, schema_name, category)
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
    ObjectCategory {
        connection_id: String,
        schema: String,
        category: ObjectCategory,
    },
    Table {
        connection_id: String,
        schema: String,
        table: String,
    },
    View {
        connection_id: String,
        schema: String,
        view: String,
    },
    Function {
        connection_id: String,
        schema: String,
        function: String,
    },
    Trigger {
        connection_id: String,
        schema: String,
        trigger: String,
    },
    Sequence {
        connection_id: String,
        schema: String,
        sequence: String,
    },
    Index {
        connection_id: String,
        schema: String,
        index: String,
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

                ui.separator();

                // Search toggle button
                let search_icon = if self.show_search { "🔍✖" } else { "🔍" };
                if ui.button(search_icon).clicked() {
                    self.show_search = !self.show_search;
                    if !self.show_search {
                        self.search_text.clear();
                    }
                }
            });

            // Search bar
            if self.show_search {
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    let response = ui.text_edit_singleline(&mut self.search_text);
                    if response.changed() {
                        // Search text changed - could trigger filtering here
                    }
                    if ui.button("Clear").clicked() {
                        self.search_text.clear();
                    }
                });
            }

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
                    .id_salt("database_tree_scroll")
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
            .id_salt(format!("saved_connection_{}", saved_connection.id))
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
                ui.close_kind(egui::UiKind::Menu);
            }

            if ui.button("✏️ Edit").clicked() {
                self.pending_action = Some((ConnectionAction::Edit, saved_connection.id.clone()));
                ui.close_kind(egui::UiKind::Menu);
            }

            if ui.button("📋 Duplicate").clicked() {
                self.pending_action =
                    Some((ConnectionAction::Duplicate, saved_connection.id.clone()));
                ui.close_kind(egui::UiKind::Menu);
            }

            if ui.button("📄 Copy Connection URL").clicked() {
                self.pending_action =
                    Some((ConnectionAction::CopyUrl, saved_connection.id.clone()));
                ui.close_kind(egui::UiKind::Menu);
            }

            ui.separator();

            if ui.button("🗑️ Delete").clicked() {
                self.pending_action = Some((ConnectionAction::Delete, saved_connection.id.clone()));
                ui.close_kind(egui::UiKind::Menu);
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
            .id_salt(format!("connection_{}", connection_id))
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

        // Get object counts for this schema
        let counts = self
            .connections
            .get(connection_id)
            .and_then(|conn| conn.object_counts.get(&schema.name))
            .cloned()
            .unwrap_or_default();

        let total_objects = counts.tables
            + counts.views
            + counts.materialized_views
            + counts.functions
            + counts.procedures
            + counts.triggers
            + counts.sequences
            + counts.indexes;

        let schema_label = if total_objects > 0 {
            format!("📁 {} ({})", schema.name, total_objects)
        } else {
            format!("📁 {}", schema.name)
        };

        let header_response = CollapsingHeader::new(schema_label)
            .id_salt(format!("schema_{}_{}", connection_id, schema.name))
            .default_open(schema_expanded)
            .show(ui, |ui| {
                // Show object categories
                self.render_object_category(
                    ui,
                    connection_id,
                    &schema.name,
                    ObjectCategory::Tables,
                    &counts,
                );
                self.render_object_category(
                    ui,
                    connection_id,
                    &schema.name,
                    ObjectCategory::Views,
                    &counts,
                );
                self.render_object_category(
                    ui,
                    connection_id,
                    &schema.name,
                    ObjectCategory::Functions,
                    &counts,
                );
                self.render_object_category(
                    ui,
                    connection_id,
                    &schema.name,
                    ObjectCategory::Triggers,
                    &counts,
                );
                self.render_object_category(
                    ui,
                    connection_id,
                    &schema.name,
                    ObjectCategory::Sequences,
                    &counts,
                );
                self.render_object_category(
                    ui,
                    connection_id,
                    &schema.name,
                    ObjectCategory::Indexes,
                    &counts,
                );
            });

        // Track expansion state
        if header_response.header_response.clicked() {
            let new_state = !schema_expanded;
            self.expanded_schemas.insert(schema_key, new_state);

            // Load object counts if expanding for the first time
            if let Some(connection) = self.connections.get(connection_id) {
                if new_state && !connection.object_counts.contains_key(&schema.name) {
                    // Request object counts for this schema
                    // This will be handled by the main application
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

    fn render_object_category(
        &mut self,
        ui: &mut Ui,
        connection_id: &str,
        schema_name: &str,
        category: ObjectCategory,
        counts: &ObjectCounts,
    ) {
        let (icon, label, count) = match category {
            ObjectCategory::Tables => ("📋", "Tables", counts.tables),
            ObjectCategory::Views => ("👁", "Views", counts.views + counts.materialized_views),
            ObjectCategory::Functions => ("⚙️", "Functions", counts.functions + counts.procedures),
            ObjectCategory::Triggers => ("⚡", "Triggers", counts.triggers),
            ObjectCategory::Sequences => ("🔢", "Sequences", counts.sequences),
            ObjectCategory::Indexes => ("🗂️", "Indexes", counts.indexes),
            ObjectCategory::SystemCatalog => ("🔧", "System Catalog", 0),
        };

        // Skip categories with no objects
        if count == 0 {
            return;
        }

        // Skip categories with no matches when searching
        if !self.search_text.is_empty()
            && !self.category_has_matches(connection_id, schema_name, &category)
        {
            return;
        }

        let category_key = format!("{}.{}.{:?}", connection_id, schema_name, category);
        let category_expanded = self
            .expanded_object_categories
            .get(&category_key)
            .copied()
            .unwrap_or(false);

        let category_label = format!("{} {} ({})", icon, label, count);

        let header_response = CollapsingHeader::new(category_label)
            .id_salt(format!(
                "category_{}_{}_{:?}",
                connection_id, schema_name, category
            ))
            .default_open(category_expanded)
            .show(ui, |ui| {
                self.render_objects_in_category(ui, connection_id, schema_name, &category);
            });

        // Track expansion state
        if header_response.header_response.clicked() {
            let new_state = !category_expanded;
            self.expanded_object_categories
                .insert(category_key, new_state);

            // Load objects if expanding for the first time
            if new_state {
                match &category {
                    ObjectCategory::Tables => {
                        // Use existing table loading mechanism
                        if let Some(connection) = self.connections.get(connection_id) {
                            if !connection.tables.contains_key(schema_name) {
                                self.schemas_needing_tables
                                    .push((connection_id.to_string(), schema_name.to_string()));
                            }
                        }
                    }
                    _ => {
                        // Use new object loading mechanism for other types
                        self.schemas_needing_objects.push((
                            connection_id.to_string(),
                            schema_name.to_string(),
                            category.clone(),
                        ));
                    }
                }
            }
        }

        // Handle category selection
        if header_response.header_response.secondary_clicked() {
            self.selected_item = Some(TreeItem::ObjectCategory {
                connection_id: connection_id.to_string(),
                schema: schema_name.to_string(),
                category,
            });
            // TODO: Show category context menu
        }
    }

    fn render_objects_in_category(
        &mut self,
        ui: &mut Ui,
        connection_id: &str,
        schema_name: &str,
        category: &ObjectCategory,
    ) {
        if let Some(connection) = self.connections.get(connection_id) {
            match category {
                ObjectCategory::Tables => {
                    if let Some(tables) = connection.tables.get(schema_name).cloned() {
                        for table in &tables {
                            if self.matches_search(&table.name) {
                                self.render_table_node(ui, connection_id, schema_name, table);
                            }
                        }
                    } else {
                        ui.label("Loading tables...");
                    }
                }
                ObjectCategory::Views => {
                    if let Some(views) = connection.views.get(schema_name).cloned() {
                        for view in &views {
                            if self.matches_search(&view.name) {
                                self.render_view_node(ui, connection_id, schema_name, view);
                            }
                        }
                    } else {
                        ui.label("Loading views...");
                    }
                }
                ObjectCategory::Functions => {
                    if let Some(functions) = connection.functions.get(schema_name).cloned() {
                        for function in &functions {
                            if self.matches_search(&function.name) {
                                self.render_function_node(ui, connection_id, schema_name, function);
                            }
                        }
                    } else {
                        ui.label("Loading functions...");
                    }
                }
                ObjectCategory::Triggers => {
                    if let Some(triggers) = connection.triggers.get(schema_name).cloned() {
                        for trigger in &triggers {
                            if self.matches_search(&trigger.name) {
                                self.render_trigger_node(ui, connection_id, schema_name, trigger);
                            }
                        }
                    } else {
                        ui.label("Loading triggers...");
                    }
                }
                ObjectCategory::Sequences => {
                    if let Some(sequences) = connection.sequences.get(schema_name).cloned() {
                        for sequence in &sequences {
                            if self.matches_search(&sequence.name) {
                                self.render_sequence_node(ui, connection_id, schema_name, sequence);
                            }
                        }
                    } else {
                        ui.label("Loading sequences...");
                    }
                }
                ObjectCategory::Indexes => {
                    if let Some(indexes) = connection.indexes.get(schema_name).cloned() {
                        for index in &indexes {
                            if self.matches_search(&index.name) {
                                self.render_index_node(ui, connection_id, schema_name, index);
                            }
                        }
                    } else {
                        ui.label("Loading indexes...");
                    }
                }
                ObjectCategory::SystemCatalog => {
                    ui.label("System catalog objects...");
                }
            }
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
            .id_salt(format!(
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

    fn render_view_node(
        &mut self,
        ui: &mut Ui,
        connection_id: &str,
        schema_name: &str,
        view: &View,
    ) {
        let view_icon = match view.view_type {
            crate::database::ViewType::Materialized => "📊",
            crate::database::ViewType::Regular => "👁",
        };

        let view_text = format!("{} {}", view_icon, view.name);
        let response = ui.selectable_label(
            self.selected_item
                == Some(TreeItem::View {
                    connection_id: connection_id.to_string(),
                    schema: schema_name.to_string(),
                    view: view.name.clone(),
                }),
            view_text,
        );

        if response.clicked() {
            self.selected_item = Some(TreeItem::View {
                connection_id: connection_id.to_string(),
                schema: schema_name.to_string(),
                view: view.name.clone(),
            });
        }

        // Show view details on hover
        let mut hover_text = format!("Type: {:?}", view.view_type);
        if let Some(owner) = &view.owner {
            hover_text.push_str(&format!("\nOwner: {}", owner));
        }
        if let Some(comment) = &view.comment {
            hover_text.push_str(&format!("\nComment: {}", comment));
        }
        let response = response.on_hover_text(hover_text);

        // Handle right-click context menu
        response.context_menu(|ui| {
            if ui.button("📋 Copy Name").clicked() {
                ui.output_mut(|o| o.copied_text = view.name.clone());
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("👁 View Definition").clicked() {
                // TODO: Show view definition
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("📊 Properties").clicked() {
                // TODO: Show view properties
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("🔄 Refresh").clicked() {
                // TODO: Refresh view
                ui.close_kind(egui::UiKind::Menu);
            }
        });
    }

    fn render_function_node(
        &mut self,
        ui: &mut Ui,
        connection_id: &str,
        schema_name: &str,
        function: &Function,
    ) {
        let function_icon = match function.function_type {
            crate::database::FunctionType::Procedure => "📋",
            crate::database::FunctionType::Aggregate => "📊",
            crate::database::FunctionType::Window => "🪟",
            crate::database::FunctionType::Function => "⚙️",
        };

        let function_text = format!("{} {}", function_icon, function.name);
        let response = ui.selectable_label(
            self.selected_item
                == Some(TreeItem::Function {
                    connection_id: connection_id.to_string(),
                    schema: schema_name.to_string(),
                    function: function.name.clone(),
                }),
            function_text,
        );

        if response.clicked() {
            self.selected_item = Some(TreeItem::Function {
                connection_id: connection_id.to_string(),
                schema: schema_name.to_string(),
                function: function.name.clone(),
            });
        }

        // Show function details on hover
        let mut hover_text = format!(
            "Type: {:?}\nLanguage: {}\nReturn Type: {}",
            function.function_type, function.language, function.return_type
        );
        if let Some(owner) = &function.owner {
            hover_text.push_str(&format!("\nOwner: {}", owner));
        }
        if let Some(comment) = &function.comment {
            hover_text.push_str(&format!("\nComment: {}", comment));
        }
        let response = response.on_hover_text(hover_text);

        // Handle right-click context menu
        response.context_menu(|ui| {
            if ui.button("📋 Copy Name").clicked() {
                ui.output_mut(|o| o.copied_text = function.name.clone());
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("⚙️ View Definition").clicked() {
                // TODO: Show function definition
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("📊 Properties").clicked() {
                // TODO: Show function properties
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("▶️ Execute").clicked() {
                // TODO: Execute function
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("🔄 Refresh").clicked() {
                // TODO: Refresh function
                ui.close_kind(egui::UiKind::Menu);
            }
        });
    }

    fn render_trigger_node(
        &mut self,
        ui: &mut Ui,
        connection_id: &str,
        schema_name: &str,
        trigger: &Trigger,
    ) {
        let trigger_text = format!("⚡ {}", trigger.name);
        let response = ui.selectable_label(
            self.selected_item
                == Some(TreeItem::Trigger {
                    connection_id: connection_id.to_string(),
                    schema: schema_name.to_string(),
                    trigger: trigger.name.clone(),
                }),
            trigger_text,
        );

        if response.clicked() {
            self.selected_item = Some(TreeItem::Trigger {
                connection_id: connection_id.to_string(),
                schema: schema_name.to_string(),
                trigger: trigger.name.clone(),
            });
        }

        // Show trigger details on hover
        let mut hover_text = format!(
            "Table: {}\nType: {:?}\nTiming: {:?}\nFunction: {}.{}",
            trigger.table_name,
            trigger.trigger_type,
            trigger.timing,
            trigger.function_schema,
            trigger.function_name
        );
        if let Some(comment) = &trigger.comment {
            hover_text.push_str(&format!("\nComment: {}", comment));
        }
        let response = response.on_hover_text(hover_text);

        // Handle right-click context menu
        response.context_menu(|ui| {
            if ui.button("📋 Copy Name").clicked() {
                ui.output_mut(|o| o.copied_text = trigger.name.clone());
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("⚡ View Definition").clicked() {
                // TODO: Show trigger definition
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("📊 Properties").clicked() {
                // TODO: Show trigger properties
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("🔄 Refresh").clicked() {
                // TODO: Refresh trigger
                ui.close_kind(egui::UiKind::Menu);
            }
        });
    }

    fn render_sequence_node(
        &mut self,
        ui: &mut Ui,
        connection_id: &str,
        schema_name: &str,
        sequence: &Sequence,
    ) {
        let sequence_text = format!("🔢 {}", sequence.name);
        let response = ui.selectable_label(
            self.selected_item
                == Some(TreeItem::Sequence {
                    connection_id: connection_id.to_string(),
                    schema: schema_name.to_string(),
                    sequence: sequence.name.clone(),
                }),
            sequence_text,
        );

        if response.clicked() {
            self.selected_item = Some(TreeItem::Sequence {
                connection_id: connection_id.to_string(),
                schema: schema_name.to_string(),
                sequence: sequence.name.clone(),
            });
        }

        // Show sequence details on hover
        let mut hover_text = format!(
            "Type: {}\nIncrement: {}\nStart: {}",
            sequence.data_type, sequence.increment, sequence.start_value
        );
        if let Some(last_value) = sequence.last_value {
            hover_text.push_str(&format!("\nLast Value: {}", last_value));
        }
        if let Some(owner_table) = &sequence.owner_table {
            hover_text.push_str(&format!("\nOwner Table: {}", owner_table));
        }
        if let Some(comment) = &sequence.comment {
            hover_text.push_str(&format!("\nComment: {}", comment));
        }
        let response = response.on_hover_text(hover_text);

        // Handle right-click context menu
        response.context_menu(|ui| {
            if ui.button("📋 Copy Name").clicked() {
                ui.output_mut(|o| o.copied_text = sequence.name.clone());
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("🔢 View Definition").clicked() {
                // TODO: Show sequence definition
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("📊 Properties").clicked() {
                // TODO: Show sequence properties
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("⏭️ Next Value").clicked() {
                // TODO: Get next sequence value
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("🔄 Refresh").clicked() {
                // TODO: Refresh sequence
                ui.close_kind(egui::UiKind::Menu);
            }
        });
    }

    fn render_index_node(
        &mut self,
        ui: &mut Ui,
        connection_id: &str,
        schema_name: &str,
        index: &Index,
    ) {
        let index_icon = if index.is_primary {
            "🔑"
        } else if index.is_unique {
            "🔒"
        } else {
            "🗂️"
        };

        let index_text = format!("{} {}", index_icon, index.name);
        let response = ui.selectable_label(
            self.selected_item
                == Some(TreeItem::Index {
                    connection_id: connection_id.to_string(),
                    schema: schema_name.to_string(),
                    index: index.name.clone(),
                }),
            index_text,
        );

        if response.clicked() {
            self.selected_item = Some(TreeItem::Index {
                connection_id: connection_id.to_string(),
                schema: schema_name.to_string(),
                index: index.name.clone(),
            });
        }

        // Show index details on hover
        let mut hover_text = format!(
            "Table: {}\nType: {:?}\nUnique: {}\nPrimary: {}",
            index.table_name, index.index_type, index.is_unique, index.is_primary
        );
        if index.is_partial {
            hover_text.push_str("\nPartial: Yes");
        }
        if let Some(comment) = &index.comment {
            hover_text.push_str(&format!("\nComment: {}", comment));
        }
        let response = response.on_hover_text(hover_text);

        // Handle right-click context menu
        response.context_menu(|ui| {
            if ui.button("📋 Copy Name").clicked() {
                ui.output_mut(|o| o.copied_text = index.name.clone());
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("🗂️ View Definition").clicked() {
                // TODO: Show index definition
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("📊 Properties").clicked() {
                // TODO: Show index properties
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("📈 Statistics").clicked() {
                // TODO: Show index statistics
                ui.close_kind(egui::UiKind::Menu);
            }
            if ui.button("🔄 Refresh").clicked() {
                // TODO: Refresh index
                ui.close_kind(egui::UiKind::Menu);
            }
        });
    }

    /// Check if an item name matches the current search filter
    fn matches_search(&self, name: &str) -> bool {
        if self.search_text.is_empty() {
            return true;
        }
        name.to_lowercase()
            .contains(&self.search_text.to_lowercase())
    }

    /// Check if any object in a category matches the search filter
    fn category_has_matches(
        &self,
        connection_id: &str,
        schema_name: &str,
        category: &ObjectCategory,
    ) -> bool {
        if self.search_text.is_empty() {
            return true;
        }

        if let Some(connection) = self.connections.get(connection_id) {
            match category {
                ObjectCategory::Tables => {
                    if let Some(tables) = connection.tables.get(schema_name) {
                        return tables.iter().any(|table| self.matches_search(&table.name));
                    }
                }
                ObjectCategory::Views => {
                    if let Some(views) = connection.views.get(schema_name) {
                        return views.iter().any(|view| self.matches_search(&view.name));
                    }
                }
                ObjectCategory::Functions => {
                    if let Some(functions) = connection.functions.get(schema_name) {
                        return functions
                            .iter()
                            .any(|function| self.matches_search(&function.name));
                    }
                }
                ObjectCategory::Triggers => {
                    if let Some(triggers) = connection.triggers.get(schema_name) {
                        return triggers
                            .iter()
                            .any(|trigger| self.matches_search(&trigger.name));
                    }
                }
                ObjectCategory::Sequences => {
                    if let Some(sequences) = connection.sequences.get(schema_name) {
                        return sequences
                            .iter()
                            .any(|sequence| self.matches_search(&sequence.name));
                    }
                }
                ObjectCategory::Indexes => {
                    if let Some(indexes) = connection.indexes.get(schema_name) {
                        return indexes.iter().any(|index| self.matches_search(&index.name));
                    }
                }
                ObjectCategory::SystemCatalog => {
                    // TODO: Implement system catalog search
                    return false;
                }
            }
        }
        false
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

    pub fn set_views(&mut self, connection_id: &str, schema: String, views: Vec<View>) {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            connection.views.insert(schema, views);
        }
    }

    pub fn set_functions(&mut self, connection_id: &str, schema: String, functions: Vec<Function>) {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            connection.functions.insert(schema, functions);
        }
    }

    pub fn set_triggers(&mut self, connection_id: &str, schema: String, triggers: Vec<Trigger>) {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            connection.triggers.insert(schema, triggers);
        }
    }

    pub fn set_sequences(&mut self, connection_id: &str, schema: String, sequences: Vec<Sequence>) {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            connection.sequences.insert(schema, sequences);
        }
    }

    pub fn set_indexes(&mut self, connection_id: &str, schema: String, indexes: Vec<Index>) {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            connection.indexes.insert(schema, indexes);
        }
    }

    pub fn set_object_counts(&mut self, connection_id: &str, schema: String, counts: ObjectCounts) {
        if let Some(connection) = self.connections.get_mut(connection_id) {
            connection.object_counts.insert(schema, counts);
        }
    }

    pub fn clear(&mut self) {
        self.connections.clear();
        self.expanded_connections.clear();
        self.expanded_schemas.clear();
        self.expanded_object_categories.clear();
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

    pub fn get_schemas_needing_objects(&mut self) -> Vec<(String, String, ObjectCategory)> {
        let schemas = self.schemas_needing_objects.clone();
        self.schemas_needing_objects.clear();
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
            expanded_object_categories: HashMap::new(),
            expanded_tables: HashMap::new(),
            selected_item: None,

            // Search and filtering
            search_text: String::new(),
            show_search: false,

            is_loading: false,
            schemas_needing_tables: Vec::new(),
            schemas_needing_objects: Vec::new(),
            tables_needing_columns: Vec::new(),
            pending_action: None,
        }
    }
}
