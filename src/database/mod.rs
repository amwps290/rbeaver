pub mod connection;
pub mod postgresql;
pub mod postgresql_queries;
pub mod query;
pub mod traits;

// Re-export main types
pub use connection::{ConnectionManager, ConnectionParams, DatabaseType, SslMode};
pub use postgresql::PostgreSQLConnection;
pub use query::{GeometryValue, QueryColumn, QueryResult, QueryRow, QueryType, QueryValue};
pub use traits::{
    ArgumentMode, Column, Database, DatabaseConnection, DatabaseObjectCounts, Function,
    FunctionArgument, FunctionType, Index, IndexColumn, IndexType, NullsOrder, ObjectCategory,
    ObjectCounts, QueryExecutor, Schema, Sequence, SortDirection, Table, Trigger, TriggerEvent,
    TriggerTiming, TriggerType, View, ViewType,
};

// Error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Query execution failed: {0}")]
    QueryFailed(String),

    #[error("Invalid connection parameters: {0}")]
    InvalidParams(String),

    #[error("Database not connected")]
    NotConnected,

    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::Database(db_err) => DatabaseError::QueryFailed(db_err.to_string()),
            sqlx::Error::Io(io_err) => DatabaseError::ConnectionFailed(io_err.to_string()),
            sqlx::Error::Configuration(config_err) => {
                DatabaseError::InvalidParams(config_err.to_string())
            }
            _ => DatabaseError::Internal(err.to_string()),
        }
    }
}
