use crate::database::ConnectionParams;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub connections: Vec<ConnectionParams>,
    pub ui_settings: UiSettings,
    pub editor_settings: EditorSettings,
    pub general_settings: GeneralSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: String,
    pub font_size: f32,
    pub show_line_numbers: bool,
    pub show_row_numbers: bool,
    pub window_size: (f32, f32),
    pub window_position: Option<(f32, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    pub tab_size: usize,
    pub auto_indent: bool,
    pub word_wrap: bool,
    pub syntax_highlighting: bool,
    pub auto_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub auto_save_connections: bool,
    pub query_timeout: u32,
    pub max_rows_display: usize,
    pub log_level: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            connections: Vec::new(),
            ui_settings: UiSettings::default(),
            editor_settings: EditorSettings::default(),
            general_settings: GeneralSettings::default(),
        }
    }
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            font_size: 14.0,
            show_line_numbers: true,
            show_row_numbers: true,
            window_size: (1200.0, 800.0),
            window_position: None,
        }
    }
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            tab_size: 4,
            auto_indent: true,
            word_wrap: false,
            syntax_highlighting: true,
            auto_complete: true,
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            auto_save_connections: true,
            query_timeout: 30,
            max_rows_display: 1000,
            log_level: "info".to_string(),
        }
    }
}

impl AppSettings {
    /// Load settings from file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let settings: AppSettings = serde_json::from_str(&content)?;
            Ok(settings)
        } else {
            Ok(Self::default())
        }
    }

    /// Save settings to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path()?;

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// Get the configuration file path
    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not find config directory")?
            .join("rbeaver");

        Ok(config_dir.join("settings.json"))
    }

    /// Add a new connection
    pub fn add_connection(&mut self, connection: ConnectionParams) -> Result<(), String> {
        // Validate connection parameters
        connection.validate()?;

        // Check for duplicate names (excluding same ID)
        if self
            .connections
            .iter()
            .any(|c| c.name == connection.name && c.id != connection.id)
        {
            return Err(format!(
                "A connection with the name '{}' already exists",
                connection.name
            ));
        }

        // Remove existing connection with same ID
        self.connections.retain(|c| c.id != connection.id);
        self.connections.push(connection);
        Ok(())
    }

    /// Remove a connection by ID
    pub fn remove_connection(&mut self, id: &str) -> Result<ConnectionParams, String> {
        let index = self
            .connections
            .iter()
            .position(|c| c.id == id)
            .ok_or_else(|| format!("Connection with ID '{}' not found", id))?;

        Ok(self.connections.remove(index))
    }

    /// Get a connection by ID
    pub fn get_connection(&self, id: &str) -> Option<&ConnectionParams> {
        self.connections.iter().find(|c| c.id == id)
    }

    /// Get a mutable reference to a connection by ID
    pub fn get_connection_mut(&mut self, id: &str) -> Option<&mut ConnectionParams> {
        self.connections.iter_mut().find(|c| c.id == id)
    }

    /// Update a connection
    pub fn update_connection(&mut self, connection: ConnectionParams) -> Result<(), String> {
        // Validate connection parameters
        connection.validate()?;

        // Check for duplicate names (excluding same ID)
        if self
            .connections
            .iter()
            .any(|c| c.name == connection.name && c.id != connection.id)
        {
            return Err(format!(
                "A connection with the name '{}' already exists",
                connection.name
            ));
        }

        if let Some(existing) = self.connections.iter_mut().find(|c| c.id == connection.id) {
            *existing = connection;
            Ok(())
        } else {
            Err(format!("Connection with ID '{}' not found", connection.id))
        }
    }

    /// Duplicate a connection
    pub fn duplicate_connection(
        &mut self,
        id: &str,
        new_name: Option<String>,
    ) -> Result<ConnectionParams, String> {
        let original = self
            .get_connection(id)
            .ok_or_else(|| format!("Connection with ID '{}' not found", id))?;

        let mut duplicated = original.duplicate(new_name);

        // Ensure unique name
        let mut counter = 1;
        let base_name = duplicated.name.clone();
        while self.connections.iter().any(|c| c.name == duplicated.name) {
            duplicated.name = format!("{} ({})", base_name, counter);
            counter += 1;
        }

        self.connections.push(duplicated.clone());
        Ok(duplicated)
    }

    /// Get all connections
    pub fn get_all_connections(&self) -> &Vec<ConnectionParams> {
        &self.connections
    }

    /// Check if a connection name exists (excluding a specific ID)
    pub fn connection_name_exists(&self, name: &str, exclude_id: Option<&str>) -> bool {
        self.connections
            .iter()
            .any(|c| c.name == name && exclude_id.map_or(true, |id| c.id != id))
    }
}
