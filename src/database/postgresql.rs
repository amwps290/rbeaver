use crate::database::{
    Column as DbColumn, ConnectionParams, Database, DatabaseConnection, DatabaseError,
    GeometryValue, QueryColumn, QueryExecutor, QueryResult, QueryRow, QueryValue, Schema, Table,
};
use async_trait::async_trait;
use sqlx::{Column, PgPool, Row, TypeInfo, ValueRef};
use std::time::Instant;

/// PostgreSQL database connection implementation
pub struct PostgreSQLConnection {
    pool: Option<PgPool>,
    connection_info: Option<String>,
}

impl PostgreSQLConnection {
    pub fn new() -> Self {
        Self {
            pool: None,
            connection_info: None,
        }
    }

    /// Get the connection pool (internal use)
    fn get_pool(&self) -> Result<&PgPool, DatabaseError> {
        self.pool.as_ref().ok_or(DatabaseError::NotConnected)
    }
}

impl Default for PostgreSQLConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseConnection for PostgreSQLConnection {
    async fn connect(&mut self, params: &ConnectionParams) -> Result<(), DatabaseError> {
        let connection_string = params.build_connection_string();

        // Create connection pool
        let pool = PgPool::connect(&connection_string).await?;

        // Test the connection
        sqlx::query("SELECT 1").fetch_one(&pool).await?;

        self.pool = Some(pool);
        self.connection_info = Some(format!(
            "{}@{}:{}/{}",
            params.username, params.host, params.port, params.database
        ));

        log::info!(
            "Connected to PostgreSQL database: {}",
            self.connection_info.as_ref().unwrap()
        );
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), DatabaseError> {
        if let Some(pool) = self.pool.take() {
            pool.close().await;
            log::info!("Disconnected from PostgreSQL database");
        }
        self.connection_info = None;
        Ok(())
    }

    async fn is_connected(&self) -> bool {
        if let Some(pool) = &self.pool {
            !pool.is_closed()
        } else {
            false
        }
    }

    async fn test_connection(&self, params: &ConnectionParams) -> Result<(), DatabaseError> {
        let connection_string = params.build_connection_string();
        let pool = PgPool::connect(&connection_string).await?;
        sqlx::query("SELECT 1").fetch_one(&pool).await?;
        pool.close().await;
        Ok(())
    }

    fn database_type(&self) -> &'static str {
        "PostgreSQL"
    }

    fn connection_info(&self) -> Option<String> {
        self.connection_info.clone()
    }
}

#[async_trait]
impl QueryExecutor for PostgreSQLConnection {
    async fn execute_query(&self, sql: &str) -> Result<QueryResult, DatabaseError> {
        let pool = self.get_pool()?;
        let start_time = Instant::now();

        let rows = sqlx::query(sql).fetch_all(pool).await?;
        let execution_time = start_time.elapsed();

        let mut result = QueryResult::new(sql.to_string()).with_execution_time(execution_time);

        if !rows.is_empty() {
            // Extract column information from the first row
            let first_row = &rows[0];
            let columns: Vec<QueryColumn> = first_row
                .columns()
                .iter()
                .enumerate()
                .map(|(i, col)| {
                    QueryColumn::new(
                        col.name().to_string(),
                        col.type_info().name().to_string(),
                        i,
                        true, // TODO: Get actual nullable info
                    )
                })
                .collect();

            result = result.with_columns(columns);

            // Convert rows to QueryRow format
            let query_rows: Result<Vec<QueryRow>, DatabaseError> = rows
                .iter()
                .map(|row| {
                    let values: Result<Vec<QueryValue>, DatabaseError> = (0..row.len())
                        .map(|i| convert_postgres_value(row, i))
                        .collect();
                    values.map(QueryRow::new)
                })
                .collect();

            result = result.with_rows(query_rows?);
        }

        Ok(result)
    }

    async fn execute_non_query(&self, sql: &str) -> Result<u64, DatabaseError> {
        let pool = self.get_pool()?;
        let result = sqlx::query(sql).execute(pool).await?;
        Ok(result.rows_affected())
    }

    async fn get_schemas(&self) -> Result<Vec<Schema>, DatabaseError> {
        let pool = self.get_pool()?;

        let rows = sqlx::query(
            "SELECT schema_name, schema_owner 
             FROM information_schema.schemata 
             WHERE schema_name NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
             ORDER BY schema_name",
        )
        .fetch_all(pool)
        .await?;

        let schemas = rows
            .iter()
            .map(|row| Schema {
                name: row.get::<String, _>("schema_name"),
                owner: row.try_get::<String, _>("schema_owner").ok(),
            })
            .collect();

        Ok(schemas)
    }

    async fn get_tables(&self, schema: &str) -> Result<Vec<Table>, DatabaseError> {
        let pool = self.get_pool()?;

        let rows = sqlx::query(
            "SELECT
                t.table_name,
                t.table_type,
                COALESCE(obj_description(c.oid), '') as table_comment
             FROM information_schema.tables t
             LEFT JOIN pg_class c ON c.relname = t.table_name
             LEFT JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = t.table_schema
             WHERE t.table_schema = $1
             ORDER BY t.table_name",
        )
        .bind(schema)
        .fetch_all(pool)
        .await?;

        let tables = rows
            .iter()
            .map(|row| {
                let comment = row
                    .try_get::<String, _>("table_comment")
                    .ok()
                    .filter(|s| !s.is_empty());

                Table {
                    name: row.get::<String, _>("table_name"),
                    schema: schema.to_string(),
                    table_type: row.get::<String, _>("table_type"),
                    comment,
                }
            })
            .collect();

        Ok(tables)
    }

    async fn get_columns(&self, schema: &str, table: &str) -> Result<Vec<DbColumn>, DatabaseError> {
        let pool = self.get_pool()?;

        let rows = sqlx::query(
            "SELECT
                c.column_name,
                c.data_type,
                c.is_nullable,
                c.column_default,
                CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_primary_key,
                COALESCE(col_description(pgc.oid, c.ordinal_position), '') as column_comment
             FROM information_schema.columns c
             LEFT JOIN pg_class pgc ON pgc.relname = c.table_name
             LEFT JOIN pg_namespace pgn ON pgn.oid = pgc.relnamespace AND pgn.nspname = c.table_schema
             LEFT JOIN (
                 SELECT ku.column_name
                 FROM information_schema.table_constraints tc
                 JOIN information_schema.key_column_usage ku
                     ON tc.constraint_name = ku.constraint_name
                     AND tc.table_schema = ku.table_schema
                 WHERE tc.constraint_type = 'PRIMARY KEY'
                     AND tc.table_schema = $1
                     AND tc.table_name = $2
             ) pk ON c.column_name = pk.column_name
             WHERE c.table_schema = $1 AND c.table_name = $2
             ORDER BY c.ordinal_position"
        )
        .bind(schema)
        .bind(table)
        .fetch_all(pool).await?;

        let columns = rows
            .iter()
            .map(|row| {
                let comment = row
                    .try_get::<String, _>("column_comment")
                    .ok()
                    .filter(|s| !s.is_empty());

                DbColumn {
                    name: row.get::<String, _>("column_name"),
                    data_type: row.get::<String, _>("data_type"),
                    is_nullable: row.get::<String, _>("is_nullable") == "YES",
                    default_value: row.try_get::<String, _>("column_default").ok(),
                    is_primary_key: row.get::<bool, _>("is_primary_key"),
                    comment,
                }
            })
            .collect();

        Ok(columns)
    }

    async fn get_table_data(
        &self,
        schema: &str,
        table: &str,
        limit: Option<u32>,
    ) -> Result<QueryResult, DatabaseError> {
        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let sql = format!("SELECT * FROM \"{}\".\"{}\"{}", schema, table, limit_clause);
        self.execute_query(&sql).await
    }

    async fn table_exists(&self, schema: &str, table: &str) -> Result<bool, DatabaseError> {
        let pool = self.get_pool()?;

        let row = sqlx::query(
            "SELECT EXISTS (
                 SELECT 1 FROM information_schema.tables 
                 WHERE table_schema = $1 AND table_name = $2
             )",
        )
        .bind(schema)
        .bind(table)
        .fetch_one(pool)
        .await?;

        Ok(row.get::<bool, _>(0))
    }
}

impl Database for PostgreSQLConnection {
    fn clone_connection(&self) -> Box<dyn Database> {
        Box::new(PostgreSQLConnection::new())
    }
}

/// Convert PostgreSQL values to QueryValue
fn convert_postgres_value(
    row: &sqlx::postgres::PgRow,
    index: usize,
) -> Result<QueryValue, DatabaseError> {
    let column = &row.columns()[index];
    let type_name = column.type_info().name();

    // Handle NULL values first
    if let Ok(value) = row.try_get_raw(index) {
        if value.is_null() {
            return Ok(QueryValue::Null);
        }
    }

    // Convert based on PostgreSQL type
    match type_name {
        "BOOL" => Ok(QueryValue::Bool(row.get(index))),
        "INT2" | "SMALLINT" => Ok(QueryValue::Int32(row.get::<i16, _>(index) as i32)),
        "INT4" | "INTEGER" => Ok(QueryValue::Int32(row.get(index))),
        "INT8" | "BIGINT" => Ok(QueryValue::Int64(row.get(index))),
        "FLOAT4" | "REAL" => Ok(QueryValue::Float32(row.get(index))),
        "FLOAT8" | "DOUBLE PRECISION" => Ok(QueryValue::Float64(row.get(index))),
        "TEXT" | "VARCHAR" | "CHAR" | "NAME" => Ok(QueryValue::String(row.get(index))),
        "BYTEA" => Ok(QueryValue::Bytes(row.get(index))),
        "TIMESTAMP" | "TIMESTAMPTZ" => {
            let dt: chrono::DateTime<chrono::Utc> = row.get(index);
            Ok(QueryValue::DateTime(dt))
        }
        "JSON" | "JSONB" => {
            let json: serde_json::Value = row.get(index);
            Ok(QueryValue::Json(json))
        }
        "USER-DEFINED" => {
            // Handle PostGIS geometry types
            handle_user_defined_type(row, index, column)
        }
        _ => {
            // Fallback to string representation
            match row.try_get::<String, _>(index) {
                Ok(s) => Ok(QueryValue::String(s)),
                Err(_) => Ok(QueryValue::String(format!("<{}>", type_name))),
            }
        }
    }
}

/// Handle USER-DEFINED types, particularly PostGIS geometry types
fn handle_user_defined_type(
    row: &sqlx::postgres::PgRow,
    index: usize,
    column: &sqlx::postgres::PgColumn,
) -> Result<QueryValue, DatabaseError> {
    // Try to get the actual type name from the column type info
    let type_info = column.type_info();
    let type_name = type_info.name();

    // For PostGIS geometry types, we need to extract the geometry information
    // First, try to get the raw bytes
    if let Ok(bytes) = row.try_get::<Vec<u8>, _>(index) {
        // This is likely a geometry in binary format (WKB)
        return Ok(QueryValue::Geometry(GeometryValue::new(
            "GEOMETRY".to_string(),
            None, // We'll need to parse SRID from WKB if needed
            format!("<Binary Geometry Data: {} bytes>", bytes.len()),
            Some(bytes),
        )));
    }

    // Try to get as string (might be WKT format)
    if let Ok(wkt_string) = row.try_get::<String, _>(index) {
        // Parse the geometry type from WKT
        let geometry_type = extract_geometry_type_from_wkt(&wkt_string);
        let srid = extract_srid_from_wkt(&wkt_string);

        return Ok(QueryValue::Geometry(GeometryValue::new(
            geometry_type,
            srid,
            wkt_string,
            None,
        )));
    }

    // Fallback for unknown user-defined types
    Ok(QueryValue::String(format!("<USER-DEFINED: {}>", type_name)))
}

/// Extract geometry type from WKT string
fn extract_geometry_type_from_wkt(wkt: &str) -> String {
    let trimmed = wkt.trim();

    // Handle SRID prefix
    let wkt_part = if trimmed.starts_with("SRID=") {
        if let Some(semicolon_pos) = trimmed.find(';') {
            &trimmed[semicolon_pos + 1..]
        } else {
            trimmed
        }
    } else {
        trimmed
    };

    // Extract the geometry type (first word)
    if let Some(space_pos) = wkt_part.find(' ') {
        wkt_part[..space_pos].to_uppercase()
    } else if let Some(paren_pos) = wkt_part.find('(') {
        wkt_part[..paren_pos].to_uppercase()
    } else {
        "GEOMETRY".to_string()
    }
}

/// Extract SRID from WKT string
fn extract_srid_from_wkt(wkt: &str) -> Option<i32> {
    let trimmed = wkt.trim();
    if trimmed.starts_with("SRID=") {
        if let Some(semicolon_pos) = trimmed.find(';') {
            let srid_part = &trimmed[5..semicolon_pos];
            srid_part.parse().ok()
        } else {
            None
        }
    } else {
        None
    }
}
