use thiserror::Error;

/// Application-wide error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] crate::database::DatabaseError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("UI error: {0}")]
    Ui(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl AppError {
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            AppError::Database(db_err) => match db_err {
                crate::database::DatabaseError::ConnectionFailed(msg) => {
                    format!("Failed to connect to database: {}", msg)
                }
                crate::database::DatabaseError::QueryFailed(msg) => {
                    format!("Query execution failed: {}", msg)
                }
                crate::database::DatabaseError::InvalidParams(msg) => {
                    format!("Invalid connection parameters: {}", msg)
                }
                crate::database::DatabaseError::NotConnected => {
                    "Not connected to database".to_string()
                }
                crate::database::DatabaseError::UnsupportedOperation(op) => {
                    format!("Unsupported operation: {}", op)
                }
                crate::database::DatabaseError::Internal(msg) => {
                    format!("Internal database error: {}", msg)
                }
            },
            AppError::Config(msg) => format!("Configuration error: {}", msg),
            AppError::Ui(msg) => format!("UI error: {}", msg),
            AppError::Io(err) => format!("File operation failed: {}", err),
            AppError::Serialization(err) => format!("Data serialization failed: {}", err),
            AppError::Unknown(msg) => format!("Unknown error: {}", msg),
        }
    }

    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            AppError::Database(db_err) => match db_err {
                crate::database::DatabaseError::ConnectionFailed(_) => true,
                crate::database::DatabaseError::QueryFailed(_) => true,
                crate::database::DatabaseError::InvalidParams(_) => true,
                crate::database::DatabaseError::NotConnected => true,
                crate::database::DatabaseError::UnsupportedOperation(_) => false,
                crate::database::DatabaseError::Internal(_) => false,
            },
            AppError::Config(_) => true,
            AppError::Ui(_) => true,
            AppError::Io(_) => true,
            AppError::Serialization(_) => true,
            AppError::Unknown(_) => false,
        }
    }
}

/// Result type for application operations
pub type AppResult<T> = Result<T, AppError>;
