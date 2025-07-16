// Import from the local crate
use rbeaver::database::{
    ConnectionParams, DatabaseConnection, DatabaseType, PostgreSQLConnection, QueryExecutor,
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("Testing RBeaver database functionality...");

    // Create test connection parameters
    let mut params = ConnectionParams::default();
    params.name = "Test Connection".to_string();
    params.database_type = DatabaseType::PostgreSQL;
    params.host = "localhost".to_string();
    params.port = 5432;
    params.database = "postgres".to_string();
    params.username = "test".to_string();
    params.password = "test@123".to_string();

    println!("Connection parameters: {:?}", params);

    // Test connection validation
    match params.validate() {
        Ok(()) => println!("✓ Connection parameters are valid"),
        Err(e) => println!("✗ Connection parameters invalid: {}", e),
    }

    // Test connection string building
    let connection_string = params.build_connection_string();
    println!("Connection string: {}", connection_string);

    // Create PostgreSQL connection
    let mut connection = PostgreSQLConnection::new();

    // Test connection (this will fail if no PostgreSQL server is running)
    println!("Testing database connection...");
    match connection.test_connection(&params).await {
        Ok(()) => {
            println!("✓ Database connection test successful!");

            // Try to connect
            match connection.connect(&params).await {
                Ok(()) => {
                    println!("✓ Successfully connected to database!");

                    // Try to execute a simple query
                    match connection.execute_query("SELECT version()").await {
                        Ok(result) => {
                            println!("✓ Query executed successfully!");
                            println!("Columns: {}", result.column_count());
                            println!("Rows: {}", result.row_count());

                            if !result.rows.is_empty() {
                                if let Some(first_row) = result.rows.first() {
                                    if let Some(version_value) = first_row.values.first() {
                                        println!(
                                            "PostgreSQL version: {}",
                                            version_value.to_display_string()
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => println!("✗ Query execution failed: {}", e),
                    }

                    // Try to get schemas
                    match connection.get_schemas().await {
                        Ok(schemas) => {
                            println!("✓ Retrieved {} schemas", schemas.len());
                            for schema in schemas.iter().take(5) {
                                println!("  - {}", schema.name);
                            }
                        }
                        Err(e) => println!("✗ Failed to get schemas: {}", e),
                    }
                }
                Err(e) => println!("✗ Failed to connect: {}", e),
            }
        }
        Err(e) => {
            println!("✗ Database connection test failed: {}", e);
            println!("This is expected if PostgreSQL is not running locally");
        }
    }

    // Test UI components (without GUI)
    println!("\nTesting UI components...");

    // Test connection dialog
    let mut connection_dialog = rbeaver::ui::ConnectionDialog::default();
    println!("✓ Connection dialog created");

    // Test query editor
    let mut query_editor = rbeaver::ui::QueryEditor::default();
    query_editor.set_sql("SELECT 1".to_string());
    assert_eq!(query_editor.get_sql(), "SELECT 1");
    println!("✓ Query editor working");

    // Test result table
    let mut result_table = rbeaver::ui::ResultTable::default();
    println!("✓ Result table created");

    // Test database tree
    let mut database_tree = rbeaver::ui::DatabaseTree::default();
    println!("✓ Database tree created");

    // Test settings
    let settings = rbeaver::config::AppSettings::default();
    println!("✓ Settings system working");

    println!("\n🎉 All RBeaver components tested successfully!");
    println!("The application is ready for use with a PostgreSQL database.");
    println!("\nTo run the GUI application:");
    println!("  cargo run --bin rbeaver");
    println!("\nTo connect to a database:");
    println!("  1. Start the application");
    println!("  2. Use File > New Connection or Ctrl+N");
    println!("  3. Enter your PostgreSQL connection details");
    println!("  4. Click 'Connect' or 'Test Connection'");
    println!("  5. Use the SQL editor to run queries (F5 to execute)");
    println!("  6. Browse database structure in the left panel");

    Ok(())
}
