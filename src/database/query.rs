use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single value in a query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryValue {
    Null,
    Bool(bool),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    String(String),
    Bytes(Vec<u8>),
    DateTime(DateTime<Utc>),
    Json(serde_json::Value),
    Geometry(GeometryValue),
}

/// Represents PostGIS geometry data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeometryValue {
    pub geometry_type: String,
    pub srid: Option<i32>,
    pub wkt: String,
    pub binary_data: Option<Vec<u8>>,
}

impl GeometryValue {
    pub fn new(
        geometry_type: String,
        srid: Option<i32>,
        wkt: String,
        binary_data: Option<Vec<u8>>,
    ) -> Self {
        Self {
            geometry_type,
            srid,
            wkt,
            binary_data,
        }
    }

    pub fn to_display_string(&self) -> String {
        if self.wkt.len() > 100 {
            // For long WKT, show a summary
            let srid_info = self
                .srid
                .map(|s| format!("SRID={};", s))
                .unwrap_or_default();
            format!("{}{}{}...", srid_info, self.geometry_type, &self.wkt[..50])
        } else {
            // For short WKT, show the full text
            let srid_info = self
                .srid
                .map(|s| format!("SRID={};", s))
                .unwrap_or_default();
            format!("{}{}", srid_info, self.wkt)
        }
    }

    pub fn get_summary(&self) -> String {
        let srid_info = self
            .srid
            .map(|s| format!(" (SRID: {})", s))
            .unwrap_or_default();
        format!("{}{}", self.geometry_type, srid_info)
    }
}

impl QueryValue {
    /// Convert the value to a display string
    pub fn to_display_string(&self) -> String {
        match self {
            QueryValue::Null => "NULL".to_string(),
            QueryValue::Bool(b) => b.to_string(),
            QueryValue::Int32(i) => i.to_string(),
            QueryValue::Int64(i) => i.to_string(),
            QueryValue::Float32(f) => f.to_string(),
            QueryValue::Float64(f) => f.to_string(),
            QueryValue::String(s) => s.clone(),
            QueryValue::Bytes(b) => format!("\\x{}", hex::encode(b)),
            QueryValue::DateTime(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            QueryValue::Json(j) => j.to_string(),
            QueryValue::Geometry(g) => g.to_display_string(),
        }
    }

    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, QueryValue::Null)
    }
}

/// Column metadata for query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryColumn {
    pub name: String,
    pub data_type: String,
    pub ordinal: usize,
    pub nullable: bool,
}

impl QueryColumn {
    pub fn new(name: String, data_type: String, ordinal: usize, nullable: bool) -> Self {
        Self {
            name,
            data_type,
            ordinal,
            nullable,
        }
    }
}

/// A single row in a query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryRow {
    pub values: Vec<QueryValue>,
}

impl QueryRow {
    pub fn new(values: Vec<QueryValue>) -> Self {
        Self { values }
    }

    pub fn get(&self, index: usize) -> Option<&QueryValue> {
        self.values.get(index)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// Complete query result with metadata and data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<QueryColumn>,
    pub rows: Vec<QueryRow>,
    pub rows_affected: Option<u64>,
    pub execution_time: Option<std::time::Duration>,
    pub query: String,
}

impl QueryResult {
    pub fn new(query: String) -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            rows_affected: None,
            execution_time: None,
            query,
        }
    }

    pub fn with_columns(mut self, columns: Vec<QueryColumn>) -> Self {
        self.columns = columns;
        self
    }

    pub fn with_rows(mut self, rows: Vec<QueryRow>) -> Self {
        self.rows = rows;
        self
    }

    pub fn with_rows_affected(mut self, rows_affected: u64) -> Self {
        self.rows_affected = Some(rows_affected);
        self
    }

    pub fn with_execution_time(mut self, execution_time: std::time::Duration) -> Self {
        self.execution_time = Some(execution_time);
        self
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Get column by name
    pub fn get_column_by_name(&self, name: &str) -> Option<&QueryColumn> {
        self.columns.iter().find(|col| col.name == name)
    }

    /// Get column index by name
    pub fn get_column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|col| col.name == name)
    }

    /// Get value from a specific row and column
    pub fn get_value(&self, row_index: usize, column_index: usize) -> Option<&QueryValue> {
        self.rows.get(row_index)?.get(column_index)
    }

    /// Get value by row index and column name
    pub fn get_value_by_name(&self, row_index: usize, column_name: &str) -> Option<&QueryValue> {
        let column_index = self.get_column_index(column_name)?;
        self.get_value(row_index, column_index)
    }
}

/// Query execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryStats {
    pub execution_time: std::time::Duration,
    pub rows_affected: Option<u64>,
    pub rows_returned: usize,
    pub query_type: QueryType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
    Select,
    Insert,
    Update,
    Delete,
    Create,
    Drop,
    Alter,
    Other(String),
}

impl QueryType {
    pub fn from_sql(sql: &str) -> Self {
        let trimmed = sql.trim().to_uppercase();
        if trimmed.starts_with("SELECT") {
            QueryType::Select
        } else if trimmed.starts_with("INSERT") {
            QueryType::Insert
        } else if trimmed.starts_with("UPDATE") {
            QueryType::Update
        } else if trimmed.starts_with("DELETE") {
            QueryType::Delete
        } else if trimmed.starts_with("CREATE") {
            QueryType::Create
        } else if trimmed.starts_with("DROP") {
            QueryType::Drop
        } else if trimmed.starts_with("ALTER") {
            QueryType::Alter
        } else {
            QueryType::Other(
                trimmed
                    .split_whitespace()
                    .next()
                    .unwrap_or("UNKNOWN")
                    .to_string(),
            )
        }
    }
}
