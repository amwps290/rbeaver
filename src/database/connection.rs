use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Database connection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionParams {
    pub id: String,
    pub name: String,
    pub database_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub ssl_mode: SslMode,
    pub connection_timeout: Option<u32>,
    pub additional_params: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SslMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

impl Default for ConnectionParams {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: "New Connection".to_string(),
            database_type: DatabaseType::PostgreSQL,
            host: "localhost".to_string(),
            port: 5432,
            database: "postgres".to_string(),
            username: "postgres".to_string(),
            password: String::new(),
            ssl_mode: SslMode::Prefer,
            connection_timeout: Some(30),
            additional_params: HashMap::new(),
        }
    }
}

impl ConnectionParams {
    pub fn new(name: String, database_type: DatabaseType) -> Self {
        let port = match database_type {
            DatabaseType::PostgreSQL => 5432,
            DatabaseType::MySQL => 3306,
            DatabaseType::SQLite => 0, // Not applicable for SQLite
        };

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            database_type,
            port,
            ..Default::default()
        }
    }

    /// Create a duplicate of this connection with a new ID and modified name
    pub fn duplicate(&self, new_name: Option<String>) -> Self {
        let name = new_name.unwrap_or_else(|| format!("{} (Copy)", self.name));

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            database_type: self.database_type.clone(),
            host: self.host.clone(),
            port: self.port,
            database: self.database.clone(),
            username: self.username.clone(),
            password: self.password.clone(),
            ssl_mode: self.ssl_mode.clone(),
            connection_timeout: self.connection_timeout,
            additional_params: self.additional_params.clone(),
        }
    }

    /// Validate connection parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Connection name cannot be empty".to_string());
        }

        if self.host.trim().is_empty() {
            return Err("Host cannot be empty".to_string());
        }

        if self.database.trim().is_empty() && self.database_type != DatabaseType::SQLite {
            return Err("Database name cannot be empty".to_string());
        }

        if self.username.trim().is_empty() && self.database_type != DatabaseType::SQLite {
            return Err("Username cannot be empty".to_string());
        }

        if self.port == 0 && self.database_type != DatabaseType::SQLite {
            return Err("Port must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Get a display string for the connection
    pub fn get_display_string(&self) -> String {
        match self.database_type {
            DatabaseType::SQLite => format!("{} (SQLite: {})", self.name, self.database),
            _ => format!(
                "{} ({}@{}:{})",
                self.name, self.username, self.host, self.port
            ),
        }
    }

    /// Get connection info for display
    pub fn get_connection_info(&self) -> String {
        match self.database_type {
            DatabaseType::SQLite => format!("SQLite: {}", self.database),
            _ => format!(
                "{}@{}:{}/{}",
                self.username, self.host, self.port, self.database
            ),
        }
    }

    /// Generate a proper database connection URL for copying to clipboard
    pub fn get_connection_url(&self) -> String {
        match self.database_type {
            DatabaseType::PostgreSQL => {
                if self.password.is_empty() {
                    format!(
                        "postgresql://{}@{}:{}/{}",
                        self.username, self.host, self.port, self.database
                    )
                } else {
                    format!(
                        "postgresql://{}:{}@{}:{}/{}",
                        self.username, self.password, self.host, self.port, self.database
                    )
                }
            }
            DatabaseType::MySQL => {
                if self.password.is_empty() {
                    format!(
                        "mysql://{}@{}:{}/{}",
                        self.username, self.host, self.port, self.database
                    )
                } else {
                    format!(
                        "mysql://{}:{}@{}:{}/{}",
                        self.username, self.password, self.host, self.port, self.database
                    )
                }
            }
            DatabaseType::SQLite => {
                // SQLite URLs: sqlite:/// + path (always use three slashes)
                format!("sqlite:///{}", self.database.trim_start_matches('/'))
            }
        }
    }

    /// Build connection string for the database
    pub fn build_connection_string(&self) -> String {
        match self.database_type {
            DatabaseType::PostgreSQL => {
                let ssl_mode = match self.ssl_mode {
                    SslMode::Disable => "disable",
                    SslMode::Allow => "allow",
                    SslMode::Prefer => "prefer",
                    SslMode::Require => "require",
                    SslMode::VerifyCa => "verify-ca",
                    SslMode::VerifyFull => "verify-full",
                };

                format!(
                    "postgresql://{}:{}@{}:{}/{}?sslmode={}",
                    self.username, self.password, self.host, self.port, self.database, ssl_mode
                )
            }
            DatabaseType::MySQL => {
                format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.username, self.password, self.host, self.port, self.database
                )
            }
            DatabaseType::SQLite => {
                format!("sqlite://{}", self.database)
            }
        }
    }
}

/// Connection manager for handling multiple database connections
pub struct ConnectionManager {
    connections: HashMap<String, ConnectionParams>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_connection(&mut self, params: ConnectionParams) {
        self.connections.insert(params.id.clone(), params);
    }

    pub fn remove_connection(&mut self, id: &str) -> Option<ConnectionParams> {
        self.connections.remove(id)
    }

    pub fn get_connection(&self, id: &str) -> Option<&ConnectionParams> {
        self.connections.get(id)
    }

    pub fn get_connection_mut(&mut self, id: &str) -> Option<&mut ConnectionParams> {
        self.connections.get_mut(id)
    }

    pub fn list_connections(&self) -> Vec<&ConnectionParams> {
        self.connections.values().collect()
    }

    pub fn update_connection(&mut self, params: ConnectionParams) -> Result<(), String> {
        if self.connections.contains_key(&params.id) {
            self.connections.insert(params.id.clone(), params);
            Ok(())
        } else {
            Err("Connection not found".to_string())
        }
    }
}
