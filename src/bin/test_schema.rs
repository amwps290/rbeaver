use rbeaver::database::{
    ConnectionParams, DatabaseConnection, DatabaseType, PostgreSQLConnection, QueryExecutor,
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("Testing RBeaver schema introspection...");

    // Create test connection parameters
    let mut params = ConnectionParams::default();
    params.name = "Schema Test Connection".to_string();
    params.database_type = DatabaseType::PostgreSQL;
    params.host = "localhost".to_string();
    params.port = 5432;
    params.database = "postgres".to_string();
    params.username = "test".to_string();
    params.password = "test@123".to_string();

    // Create PostgreSQL connection
    let mut connection = PostgreSQLConnection::new();

    // Test connection
    match connection.connect(&params).await {
        Ok(()) => {
            println!("✓ Connected to database successfully!");

            // Test schema loading
            match connection.get_schemas().await {
                Ok(schemas) => {
                    println!("✓ Retrieved {} schemas", schemas.len());

                    // Test table loading for each schema
                    for schema in schemas.iter().take(3) {
                        // Test first 3 schemas
                        println!("\n📁 Schema: {}", schema.name);

                        match connection.get_tables(&schema.name).await {
                            Ok(tables) => {
                                println!("  ✓ Retrieved {} tables", tables.len());

                                // Test column loading for first few tables
                                for table in tables.iter().take(2) {
                                    // Test first 2 tables
                                    println!("    📋 Table: {}", table.name);

                                    match connection.get_columns(&schema.name, &table.name).await {
                                        Ok(columns) => {
                                            println!("      ✓ Retrieved {} columns", columns.len());
                                            for column in columns.iter().take(3) {
                                                // Show first 3 columns
                                                println!(
                                                    "        🔹 {}: {} ({})",
                                                    column.name,
                                                    column.data_type,
                                                    if column.is_nullable {
                                                        "nullable"
                                                    } else {
                                                        "not null"
                                                    }
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            println!("      ❌ Failed to load columns: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                println!("  ❌ Failed to load tables: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to load schemas: {}", e);
                }
            }

            // Test a simple query
            println!("\n🔍 Testing query execution...");
            match connection
                .execute_query("SELECT current_database(), current_user, version()")
                .await
            {
                Ok(result) => {
                    println!("✓ Query executed successfully!");
                    println!("  Columns: {}", result.column_count());
                    println!("  Rows: {}", result.row_count());

                    if let Some(first_row) = result.rows.first() {
                        if let Some(db_value) = first_row.values.get(0) {
                            println!("  Current database: {}", db_value.to_display_string());
                        }
                        if let Some(user_value) = first_row.values.get(1) {
                            println!("  Current user: {}", user_value.to_display_string());
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Query execution failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to connect: {}", e);
            println!("This is expected if PostgreSQL is not running locally");
        }
    }

    println!("\n🎉 Schema introspection test completed!");
    Ok(())
}
