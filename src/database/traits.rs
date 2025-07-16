use crate::database::{ConnectionParams, DatabaseError, QueryResult};
use async_trait::async_trait;

/// Schema information
#[derive(Debug, Clone)]
pub struct Schema {
    pub name: String,
    pub owner: Option<String>,
}

/// Table information
#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub schema: String,
    pub table_type: String, // TABLE, VIEW, etc.
    pub comment: Option<String>,
}

/// Column information
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub comment: Option<String>,
}

/// Core trait for database connections
#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    /// Connect to the database using the provided parameters
    async fn connect(&mut self, params: &ConnectionParams) -> Result<(), DatabaseError>;

    /// Disconnect from the database
    async fn disconnect(&mut self) -> Result<(), DatabaseError>;

    /// Check if currently connected to the database
    async fn is_connected(&self) -> bool;

    /// Test connection without establishing a persistent connection
    async fn test_connection(&self, params: &ConnectionParams) -> Result<(), DatabaseError>;

    /// Get the database type (PostgreSQL, MySQL, etc.)
    fn database_type(&self) -> &'static str;

    /// Get connection information
    fn connection_info(&self) -> Option<String>;
}

/// Trait for executing queries and retrieving database metadata
#[async_trait]
pub trait QueryExecutor: Send + Sync {
    /// Execute a SQL query and return results
    async fn execute_query(&self, sql: &str) -> Result<QueryResult, DatabaseError>;

    /// Execute a query that doesn't return data (INSERT, UPDATE, DELETE, etc.)
    async fn execute_non_query(&self, sql: &str) -> Result<u64, DatabaseError>;

    /// Get list of schemas in the database
    async fn get_schemas(&self) -> Result<Vec<Schema>, DatabaseError>;

    /// Get list of tables in a specific schema
    async fn get_tables(&self, schema: &str) -> Result<Vec<Table>, DatabaseError>;

    /// Get list of columns for a specific table
    async fn get_columns(&self, schema: &str, table: &str) -> Result<Vec<Column>, DatabaseError>;

    /// Get table data with optional limit
    async fn get_table_data(
        &self,
        schema: &str,
        table: &str,
        limit: Option<u32>,
    ) -> Result<QueryResult, DatabaseError>;

    /// Check if a table exists
    async fn table_exists(&self, schema: &str, table: &str) -> Result<bool, DatabaseError>;
}

/// Combined trait for full database functionality
pub trait Database: DatabaseConnection + QueryExecutor + Send + Sync {
    /// Clone the database connection
    fn clone_connection(&self) -> Box<dyn Database>;
}
