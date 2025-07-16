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

/// View information
#[derive(Debug, Clone)]
pub struct View {
    pub name: String,
    pub schema: String,
    pub view_type: ViewType,
    pub definition: Option<String>,
    pub comment: Option<String>,
    pub owner: Option<String>,
    pub is_updatable: bool,
}

/// View type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ViewType {
    Regular,
    Materialized,
}

/// Function information
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub schema: String,
    pub function_type: FunctionType,
    pub return_type: String,
    pub arguments: Vec<FunctionArgument>,
    pub language: String,
    pub definition: Option<String>,
    pub comment: Option<String>,
    pub owner: Option<String>,
}

/// Function type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionType {
    Function,
    Procedure,
    Aggregate,
    Window,
}

/// Function argument information
#[derive(Debug, Clone)]
pub struct FunctionArgument {
    pub name: Option<String>,
    pub data_type: String,
    pub mode: ArgumentMode,
    pub default_value: Option<String>,
}

/// Function argument mode
#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentMode {
    In,
    Out,
    InOut,
    Variadic,
}

/// Trigger information
#[derive(Debug, Clone)]
pub struct Trigger {
    pub name: String,
    pub schema: String,
    pub table_name: String,
    pub trigger_type: TriggerType,
    pub events: Vec<TriggerEvent>,
    pub timing: TriggerTiming,
    pub function_name: String,
    pub function_schema: String,
    pub condition: Option<String>,
    pub comment: Option<String>,
}

/// Trigger type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerType {
    Row,
    Statement,
}

/// Trigger event enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
    Truncate,
}

/// Trigger timing enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum TriggerTiming {
    Before,
    After,
    InsteadOf,
}

/// Sequence information
#[derive(Debug, Clone)]
pub struct Sequence {
    pub name: String,
    pub schema: String,
    pub data_type: String,
    pub start_value: i64,
    pub min_value: Option<i64>,
    pub max_value: Option<i64>,
    pub increment: i64,
    pub cycle: bool,
    pub cache_size: i64,
    pub last_value: Option<i64>,
    pub owner_table: Option<String>,
    pub owner_column: Option<String>,
    pub comment: Option<String>,
}

/// Index information
#[derive(Debug, Clone)]
pub struct Index {
    pub name: String,
    pub schema: String,
    pub table_name: String,
    pub index_type: IndexType,
    pub columns: Vec<IndexColumn>,
    pub is_unique: bool,
    pub is_primary: bool,
    pub is_partial: bool,
    pub condition: Option<String>,
    pub size: Option<i64>,
    pub comment: Option<String>,
}

/// Index type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum IndexType {
    BTree,
    Hash,
    Gist,
    Gin,
    Spgist,
    Brin,
}

/// Index column information
#[derive(Debug, Clone)]
pub struct IndexColumn {
    pub name: String,
    pub position: i32,
    pub direction: Option<SortDirection>,
    pub nulls_order: Option<NullsOrder>,
}

/// Sort direction enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Nulls order enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum NullsOrder {
    First,
    Last,
}

/// Object counts for a specific schema
#[derive(Debug, Clone, Default)]
pub struct ObjectCounts {
    pub tables: usize,
    pub views: usize,
    pub materialized_views: usize,
    pub functions: usize,
    pub procedures: usize,
    pub triggers: usize,
    pub sequences: usize,
    pub indexes: usize,
}

/// Database-wide object counts
#[derive(Debug, Clone, Default)]
pub struct DatabaseObjectCounts {
    pub schemas: usize,
    pub user_schemas: usize,
    pub system_schemas: usize,
    pub total_tables: usize,
    pub total_views: usize,
    pub total_materialized_views: usize,
    pub total_functions: usize,
    pub total_procedures: usize,
    pub total_triggers: usize,
    pub total_sequences: usize,
    pub total_indexes: usize,
}

/// Database object category for tree organization
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectCategory {
    Tables,
    Views,
    Functions,
    Triggers,
    Sequences,
    Indexes,
    SystemCatalog,
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

    /// Get list of views in a specific schema
    async fn get_views(&self, schema: &str) -> Result<Vec<View>, DatabaseError>;

    /// Get list of functions in a specific schema
    async fn get_functions(&self, schema: &str) -> Result<Vec<Function>, DatabaseError>;

    /// Get list of triggers in a specific schema
    async fn get_triggers(&self, schema: &str) -> Result<Vec<Trigger>, DatabaseError>;

    /// Get list of sequences in a specific schema
    async fn get_sequences(&self, schema: &str) -> Result<Vec<Sequence>, DatabaseError>;

    /// Get list of indexes in a specific schema
    async fn get_indexes(&self, schema: &str) -> Result<Vec<Index>, DatabaseError>;

    /// Get all schemas including system schemas
    async fn get_all_schemas(&self) -> Result<Vec<Schema>, DatabaseError>;

    /// Get object counts for a schema
    async fn get_object_counts(&self, schema: &str) -> Result<ObjectCounts, DatabaseError>;

    /// Get database-wide object counts
    async fn get_database_object_counts(&self) -> Result<DatabaseObjectCounts, DatabaseError>;
}

/// Combined trait for full database functionality
pub trait Database: DatabaseConnection + QueryExecutor + Send + Sync {
    /// Clone the database connection
    fn clone_connection(&self) -> Box<dyn Database>;
}
