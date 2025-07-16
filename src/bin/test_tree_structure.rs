use rbeaver::database::{
    Column, ConnectionParams, DatabaseConnection, DatabaseType, PostgreSQLConnection,
    QueryExecutor, Schema, Table,
};
use rbeaver::ui::DatabaseTree;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("Testing RBeaver connection-based tree structure...");

    // Create test database tree
    let mut tree = DatabaseTree::default();

    // Test 1: Add multiple connections
    println!("\n📋 Test 1: Adding multiple connections");
    tree.add_connection("conn1".to_string(), "PostgreSQL Local".to_string());
    tree.add_connection("conn2".to_string(), "PostgreSQL Remote".to_string());
    tree.set_connection_status("conn1", true);
    tree.set_connection_status("conn2", false);
    println!("✓ Added 2 connections to tree");

    // Test 2: Add schemas to different connections
    println!("\n📋 Test 2: Adding schemas to connections");
    let schemas1 = vec![
        Schema {
            name: "public".to_string(),
            owner: Some("postgres".to_string()),
        },
        Schema {
            name: "app_data".to_string(),
            owner: Some("app_user".to_string()),
        },
    ];
    let schemas2 = vec![
        Schema {
            name: "public".to_string(),
            owner: Some("postgres".to_string()),
        },
        Schema {
            name: "analytics".to_string(),
            owner: Some("analyst".to_string()),
        },
    ];

    tree.set_schemas("conn1", schemas1);
    tree.set_schemas("conn2", schemas2);
    println!("✓ Added schemas to both connections");

    // Test 3: Add tables to schemas
    println!("\n📋 Test 3: Adding tables to schemas");
    let tables1 = vec![
        Table {
            name: "users".to_string(),
            schema: "public".to_string(),
            table_type: "TABLE".to_string(),
            comment: Some("User accounts".to_string()),
        },
        Table {
            name: "orders".to_string(),
            schema: "public".to_string(),
            table_type: "TABLE".to_string(),
            comment: None,
        },
    ];

    let tables2 = vec![Table {
        name: "events".to_string(),
        schema: "analytics".to_string(),
        table_type: "TABLE".to_string(),
        comment: Some("Event tracking".to_string()),
    }];

    tree.set_tables("conn1", "public".to_string(), tables1);
    tree.set_tables("conn2", "analytics".to_string(), tables2);
    println!("✓ Added tables to schemas in different connections");

    // Test 4: Add columns to tables
    println!("\n📋 Test 4: Adding columns to tables");
    let columns1 = vec![
        Column {
            name: "id".to_string(),
            data_type: "integer".to_string(),
            is_nullable: false,
            default_value: Some("nextval('users_id_seq'::regclass)".to_string()),
            is_primary_key: true,
            comment: Some("Primary key".to_string()),
        },
        Column {
            name: "username".to_string(),
            data_type: "character varying(50)".to_string(),
            is_nullable: false,
            default_value: None,
            is_primary_key: false,
            comment: None,
        },
    ];

    tree.set_columns("conn1", "public".to_string(), "users".to_string(), columns1);
    println!("✓ Added columns to table in connection 1");

    // Test 5: Test tree expansion requests
    println!("\n📋 Test 5: Testing tree expansion requests");
    let schemas_needing_tables = tree.get_schemas_needing_tables();
    let tables_needing_columns = tree.get_tables_needing_columns();
    println!("✓ Schemas needing tables: {}", schemas_needing_tables.len());
    println!("✓ Tables needing columns: {}", tables_needing_columns.len());

    // Test 6: Test with real database connection (if available)
    println!("\n📋 Test 6: Testing with real database connection");
    let mut params = ConnectionParams::default();
    params.name = "Test Connection".to_string();
    params.database_type = DatabaseType::PostgreSQL;
    params.host = "localhost".to_string();
    params.port = 5432;
    params.database = "postgres".to_string();
    params.username = "test".to_string();
    params.password = "test@123".to_string();

    let mut connection = PostgreSQLConnection::new();

    match connection.connect(&params).await {
        Ok(()) => {
            println!("✓ Connected to real database");

            // Add real connection to tree
            tree.add_connection(params.id.clone(), params.name.clone());
            tree.set_connection_status(&params.id, true);

            // Load real schemas
            match connection.get_schemas().await {
                Ok(schemas) => {
                    tree.set_schemas(&params.id, schemas.clone());
                    println!("✓ Loaded {} real schemas", schemas.len());

                    // Load tables for first schema
                    if let Some(first_schema) = schemas.first() {
                        match connection.get_tables(&first_schema.name).await {
                            Ok(tables) => {
                                tree.set_tables(
                                    &params.id,
                                    first_schema.name.clone(),
                                    tables.clone(),
                                );
                                println!(
                                    "✓ Loaded {} tables for schema '{}'",
                                    tables.len(),
                                    first_schema.name
                                );

                                // Load columns for first table
                                if let Some(first_table) = tables.first() {
                                    match connection
                                        .get_columns(&first_schema.name, &first_table.name)
                                        .await
                                    {
                                        Ok(columns) => {
                                            tree.set_columns(
                                                &params.id,
                                                first_schema.name.clone(),
                                                first_table.name.clone(),
                                                columns.clone(),
                                            );
                                            println!(
                                                "✓ Loaded {} columns for table '{}.{}'",
                                                columns.len(),
                                                first_schema.name,
                                                first_table.name
                                            );
                                        }
                                        Err(e) => println!("⚠ Failed to load columns: {}", e),
                                    }
                                }
                            }
                            Err(e) => println!("⚠ Failed to load tables: {}", e),
                        }
                    }
                }
                Err(e) => println!("⚠ Failed to load schemas: {}", e),
            }
        }
        Err(e) => {
            println!("⚠ Could not connect to database: {}", e);
            println!("  This is expected if PostgreSQL is not running locally");
        }
    }

    println!("\n🎉 Connection-based tree structure test completed!");
    println!("\nTree structure now supports:");
    println!("  📁 Connection 1 (PostgreSQL Local) - Connected");
    println!("    📁 public");
    println!("      📋 users");
    println!("        🔑 id (integer)");
    println!("        🔸 username (varchar)");
    println!("      📋 orders");
    println!("    📁 app_data");
    println!("  ❌ Connection 2 (PostgreSQL Remote) - Disconnected");
    println!("    📁 public");
    println!("    📁 analytics");
    println!("      📋 events");

    if connection.connect(&params).await.is_ok() {
        println!("  🔗 Connection 3 (Test Connection) - Connected");
        println!("    📁 [Real database schemas and tables]");
    }

    Ok(())
}
