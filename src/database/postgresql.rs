use crate::database::{
    ArgumentMode, Column as DbColumn, ConnectionParams, Database, DatabaseConnection,
    DatabaseError, DatabaseObjectCounts, Function, FunctionArgument, FunctionType, GeometryValue,
    Index, IndexColumn, IndexType, NullsOrder, ObjectCounts, QueryColumn, QueryExecutor,
    QueryResult, QueryRow, QueryValue, Schema, Sequence, SortDirection, Table, Trigger,
    TriggerEvent, TriggerTiming, TriggerType, View, ViewType,
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

    async fn get_views(&self, schema: &str) -> Result<Vec<View>, DatabaseError> {
        let pool = self.get_pool()?;

        let rows = sqlx::query(crate::database::postgresql_queries::GET_VIEWS_QUERY)
            .bind(schema)
            .fetch_all(pool)
            .await?;

        let views = rows
            .iter()
            .map(|row| {
                let view_type = match row.get::<String, _>("view_type").as_str() {
                    "MATERIALIZED VIEW" => ViewType::Materialized,
                    _ => ViewType::Regular,
                };

                let comment = row
                    .try_get::<String, _>("comment")
                    .ok()
                    .filter(|s| !s.is_empty());

                View {
                    name: row.get::<String, _>("name"),
                    schema: row.get::<String, _>("schema"),
                    view_type,
                    definition: row.try_get::<String, _>("definition").ok(),
                    comment,
                    owner: row.try_get::<String, _>("owner").ok(),
                    is_updatable: row.get::<bool, _>("is_updatable"),
                }
            })
            .collect();

        Ok(views)
    }

    async fn get_functions(&self, schema: &str) -> Result<Vec<Function>, DatabaseError> {
        let pool = self.get_pool()?;

        let rows = sqlx::query(crate::database::postgresql_queries::GET_FUNCTIONS_QUERY)
            .bind(schema)
            .fetch_all(pool)
            .await?;

        let functions = rows
            .iter()
            .map(|row| {
                let function_type = match row.get::<String, _>("function_type").as_str() {
                    "PROCEDURE" => FunctionType::Procedure,
                    "AGGREGATE" => FunctionType::Aggregate,
                    "WINDOW" => FunctionType::Window,
                    _ => FunctionType::Function,
                };

                let comment = row
                    .try_get::<String, _>("comment")
                    .ok()
                    .filter(|s| !s.is_empty());

                // Parse arguments - simplified for now
                let arguments = parse_function_arguments(
                    &row.try_get::<String, _>("arguments").unwrap_or_default(),
                );

                Function {
                    name: row.get::<String, _>("name"),
                    schema: row.get::<String, _>("schema"),
                    function_type,
                    return_type: row.get::<String, _>("return_type"),
                    arguments,
                    language: row.get::<String, _>("language"),
                    definition: row.try_get::<String, _>("definition").ok(),
                    comment,
                    owner: row.try_get::<String, _>("owner").ok(),
                }
            })
            .collect();

        Ok(functions)
    }

    async fn get_triggers(&self, schema: &str) -> Result<Vec<Trigger>, DatabaseError> {
        let pool = self.get_pool()?;

        let rows = sqlx::query(crate::database::postgresql_queries::GET_TRIGGERS_QUERY)
            .bind(schema)
            .fetch_all(pool)
            .await?;

        let triggers = rows
            .iter()
            .map(|row| {
                let trigger_type = match row.get::<String, _>("trigger_type").as_str() {
                    "ROW" => TriggerType::Row,
                    _ => TriggerType::Statement,
                };

                let timing = match row.get::<String, _>("timing").as_str() {
                    "BEFORE" => TriggerTiming::Before,
                    "INSTEAD_OF" => TriggerTiming::InsteadOf,
                    _ => TriggerTiming::After,
                };

                let events =
                    parse_trigger_events(&row.try_get::<String, _>("events").unwrap_or_default());

                let comment = row
                    .try_get::<String, _>("comment")
                    .ok()
                    .filter(|s| !s.is_empty());

                Trigger {
                    name: row.get::<String, _>("name"),
                    schema: row.get::<String, _>("schema"),
                    table_name: row.get::<String, _>("table_name"),
                    trigger_type,
                    events,
                    timing,
                    function_name: row.get::<String, _>("function_name"),
                    function_schema: row.get::<String, _>("function_schema"),
                    condition: row.try_get::<String, _>("condition").ok(),
                    comment,
                }
            })
            .collect();

        Ok(triggers)
    }

    async fn get_sequences(&self, schema: &str) -> Result<Vec<Sequence>, DatabaseError> {
        let pool = self.get_pool()?;

        let rows = sqlx::query(crate::database::postgresql_queries::GET_SEQUENCES_QUERY)
            .bind(schema)
            .fetch_all(pool)
            .await?;

        let sequences = rows
            .iter()
            .map(|row| {
                let comment = row
                    .try_get::<String, _>("comment")
                    .ok()
                    .filter(|s| !s.is_empty());

                Sequence {
                    name: row.get::<String, _>("name"),
                    schema: row.get::<String, _>("schema"),
                    data_type: row.get::<String, _>("data_type"),
                    start_value: row.get::<i64, _>("start_value"),
                    min_value: row.try_get::<i64, _>("min_value").ok(),
                    max_value: row.try_get::<i64, _>("max_value").ok(),
                    increment: row.get::<i64, _>("increment"),
                    cycle: row.get::<bool, _>("cycle"),
                    cache_size: row.get::<i64, _>("cache_size"),
                    last_value: row.try_get::<i64, _>("last_value").ok(),
                    owner_table: row.try_get::<String, _>("owner_table").ok(),
                    owner_column: row.try_get::<String, _>("owner_column").ok(),
                    comment,
                }
            })
            .collect();

        Ok(sequences)
    }

    async fn get_indexes(&self, schema: &str) -> Result<Vec<Index>, DatabaseError> {
        let pool = self.get_pool()?;

        let rows = sqlx::query(crate::database::postgresql_queries::GET_INDEXES_QUERY)
            .bind(schema)
            .fetch_all(pool)
            .await?;

        let indexes = rows
            .iter()
            .map(|row| {
                let index_type = match row.get::<String, _>("index_type").as_str() {
                    "hash" => IndexType::Hash,
                    "gist" => IndexType::Gist,
                    "gin" => IndexType::Gin,
                    "spgist" => IndexType::Spgist,
                    "brin" => IndexType::Brin,
                    _ => IndexType::BTree,
                };

                let columns =
                    parse_index_columns(&row.try_get::<String, _>("columns").unwrap_or_default());

                let comment = row
                    .try_get::<String, _>("comment")
                    .ok()
                    .filter(|s| !s.is_empty());

                Index {
                    name: row.get::<String, _>("name"),
                    schema: row.get::<String, _>("schema"),
                    table_name: row.get::<String, _>("table_name"),
                    index_type,
                    columns,
                    is_unique: row.get::<bool, _>("is_unique"),
                    is_primary: row.get::<bool, _>("is_primary"),
                    is_partial: row.get::<bool, _>("is_partial"),
                    condition: row.try_get::<String, _>("condition").ok(),
                    size: None, // TODO: Parse size from string
                    comment,
                }
            })
            .collect();

        Ok(indexes)
    }

    async fn get_all_schemas(&self) -> Result<Vec<Schema>, DatabaseError> {
        let pool = self.get_pool()?;

        let rows = sqlx::query(crate::database::postgresql_queries::GET_ALL_SCHEMAS_QUERY)
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

    async fn get_object_counts(&self, schema: &str) -> Result<ObjectCounts, DatabaseError> {
        let pool = self.get_pool()?;

        let row = sqlx::query(crate::database::postgresql_queries::GET_OBJECT_COUNTS_QUERY)
            .bind(schema)
            .fetch_one(pool)
            .await?;

        Ok(ObjectCounts {
            tables: row.get::<i64, _>("tables") as usize,
            views: row.get::<i64, _>("views") as usize,
            materialized_views: row.get::<i64, _>("materialized_views") as usize,
            functions: row.get::<i64, _>("functions") as usize,
            procedures: row.get::<i64, _>("procedures") as usize,
            triggers: row.get::<i64, _>("triggers") as usize,
            sequences: row.get::<i64, _>("sequences") as usize,
            indexes: row.get::<i64, _>("indexes") as usize,
        })
    }

    async fn get_database_object_counts(&self) -> Result<DatabaseObjectCounts, DatabaseError> {
        let pool = self.get_pool()?;

        let row =
            sqlx::query(crate::database::postgresql_queries::GET_DATABASE_OBJECT_COUNTS_QUERY)
                .fetch_one(pool)
                .await?;

        Ok(DatabaseObjectCounts {
            schemas: row.get::<i64, _>("schemas") as usize,
            user_schemas: row.get::<i64, _>("user_schemas") as usize,
            system_schemas: row.get::<i64, _>("system_schemas") as usize,
            total_tables: row.get::<i64, _>("total_tables") as usize,
            total_views: row.get::<i64, _>("total_views") as usize,
            total_materialized_views: row.get::<i64, _>("total_materialized_views") as usize,
            total_functions: row.get::<i64, _>("total_functions") as usize,
            total_procedures: row.get::<i64, _>("total_procedures") as usize,
            total_triggers: row.get::<i64, _>("total_triggers") as usize,
            total_sequences: row.get::<i64, _>("total_sequences") as usize,
            total_indexes: row.get::<i64, _>("total_indexes") as usize,
        })
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
        "TEXT" | "VARCHAR" | "CHAR" | "NAME" => {
            let text: String = row.get(index);
            // Validate UTF-8 encoding and handle potential encoding issues
            if text.is_ascii() || std::str::from_utf8(text.as_bytes()).is_ok() {
                Ok(QueryValue::String(text))
            } else {
                log::warn!("Potential encoding issue detected in text field, attempting recovery");
                // Try to recover by replacing invalid UTF-8 sequences
                let recovered = String::from_utf8_lossy(text.as_bytes()).to_string();
                Ok(QueryValue::String(recovered))
            }
        }
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
            // Fallback to string representation with encoding validation
            match row.try_get::<String, _>(index) {
                Ok(s) => {
                    // Validate UTF-8 encoding for fallback strings
                    if s.is_ascii() || std::str::from_utf8(s.as_bytes()).is_ok() {
                        Ok(QueryValue::String(s))
                    } else {
                        log::warn!(
                            "Encoding issue in fallback string conversion for type: {}",
                            type_name
                        );
                        let recovered = String::from_utf8_lossy(s.as_bytes()).to_string();
                        Ok(QueryValue::String(recovered))
                    }
                }
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

/// Parse function arguments from PostgreSQL function signature
fn parse_function_arguments(args_str: &str) -> Vec<FunctionArgument> {
    if args_str.is_empty() {
        return Vec::new();
    }

    // Simple parsing - in a real implementation, this would be more sophisticated
    args_str
        .split(',')
        .enumerate()
        .map(|(i, arg)| {
            let trimmed = arg.trim();
            let parts: Vec<&str> = trimmed.split_whitespace().collect();

            let (name, data_type) = if parts.len() >= 2 {
                (Some(parts[0].to_string()), parts[1].to_string())
            } else if parts.len() == 1 {
                (None, parts[0].to_string())
            } else {
                (None, "unknown".to_string())
            };

            FunctionArgument {
                name,
                data_type,
                mode: ArgumentMode::In, // Default to IN mode
                default_value: None,
            }
        })
        .collect()
}

/// Parse trigger events from comma-separated string
fn parse_trigger_events(events_str: &str) -> Vec<TriggerEvent> {
    if events_str.is_empty() {
        return Vec::new();
    }

    events_str
        .split(',')
        .filter_map(|event| match event.trim().to_uppercase().as_str() {
            "INSERT" => Some(TriggerEvent::Insert),
            "UPDATE" => Some(TriggerEvent::Update),
            "DELETE" => Some(TriggerEvent::Delete),
            "TRUNCATE" => Some(TriggerEvent::Truncate),
            _ => None,
        })
        .collect()
}

/// Parse index columns from comma-separated string
fn parse_index_columns(columns_str: &str) -> Vec<IndexColumn> {
    if columns_str.is_empty() {
        return Vec::new();
    }

    columns_str
        .split(',')
        .enumerate()
        .map(|(i, column)| {
            let trimmed = column.trim();

            IndexColumn {
                name: trimmed.to_string(),
                position: i as i32 + 1,
                direction: Some(SortDirection::Ascending), // Default
                nulls_order: None,
            }
        })
        .collect()
}
