/// Test binary for database object categorization feature
///
/// This test demonstrates the new object categorization functionality
/// including hierarchical tree structure, object counts, and search capabilities.
use eframe::egui;
use rbeaver::database::{ObjectCounts, Schema};
use rbeaver::ui::DatabaseTree;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("RBeaver - Object Categorization Test"),
        ..Default::default()
    };

    eframe::run_native(
        "Object Categorization Test",
        options,
        Box::new(|_cc| Ok(Box::new(ObjectCategorizationTestApp::new()))),
    )
}

struct ObjectCategorizationTestApp {
    database_tree: DatabaseTree,
    test_connection_added: bool,
}

impl ObjectCategorizationTestApp {
    fn new() -> Self {
        let mut app = Self {
            database_tree: DatabaseTree::default(),
            test_connection_added: false,
        };

        // Add a test connection to demonstrate the feature
        app.setup_test_data();
        app
    }

    fn setup_test_data(&mut self) {
        // Create a test connection
        let connection_id = "test-connection-1".to_string();
        let connection_name = "Test PostgreSQL Database".to_string();

        self.database_tree
            .add_connection(connection_id.clone(), connection_name);

        // Add test schemas
        let schemas = vec![
            Schema {
                name: "public".to_string(),
                owner: Some("postgres".to_string()),
            },
            Schema {
                name: "inventory".to_string(),
                owner: Some("app_user".to_string()),
            },
            Schema {
                name: "analytics".to_string(),
                owner: Some("analyst".to_string()),
            },
        ];

        self.database_tree.set_schemas(&connection_id, schemas);

        // Add object counts for each schema to demonstrate the categorization
        let public_counts = ObjectCounts {
            tables: 15,
            views: 3,
            materialized_views: 1,
            functions: 8,
            procedures: 2,
            triggers: 5,
            sequences: 4,
            indexes: 25,
        };

        let inventory_counts = ObjectCounts {
            tables: 8,
            views: 2,
            materialized_views: 0,
            functions: 3,
            procedures: 1,
            triggers: 2,
            sequences: 2,
            indexes: 12,
        };

        let analytics_counts = ObjectCounts {
            tables: 5,
            views: 7,
            materialized_views: 3,
            functions: 12,
            procedures: 0,
            triggers: 1,
            sequences: 1,
            indexes: 8,
        };

        self.database_tree
            .set_object_counts(&connection_id, "public".to_string(), public_counts);
        self.database_tree.set_object_counts(
            &connection_id,
            "inventory".to_string(),
            inventory_counts,
        );
        self.database_tree.set_object_counts(
            &connection_id,
            "analytics".to_string(),
            analytics_counts,
        );

        // Add some sample tables to demonstrate the hierarchy
        let sample_tables = vec![
            rbeaver::database::Table {
                name: "users".to_string(),
                schema: "public".to_string(),
                table_type: "TABLE".to_string(),
                comment: Some("User accounts table".to_string()),
            },
            rbeaver::database::Table {
                name: "orders".to_string(),
                schema: "public".to_string(),
                table_type: "TABLE".to_string(),
                comment: Some("Customer orders".to_string()),
            },
            rbeaver::database::Table {
                name: "products".to_string(),
                schema: "public".to_string(),
                table_type: "TABLE".to_string(),
                comment: Some("Product catalog".to_string()),
            },
        ];

        self.database_tree
            .set_tables(&connection_id, "public".to_string(), sample_tables);

        // Add some sample views
        let sample_views = vec![
            rbeaver::database::View {
                name: "user_orders_view".to_string(),
                schema: "public".to_string(),
                view_type: rbeaver::database::ViewType::Regular,
                definition: Some("SELECT u.name, COUNT(o.id) as order_count FROM users u LEFT JOIN orders o ON u.id = o.user_id GROUP BY u.id, u.name".to_string()),
                comment: Some("User order summary".to_string()),
                owner: Some("postgres".to_string()),
                is_updatable: false,
            },
            rbeaver::database::View {
                name: "product_sales_summary".to_string(),
                schema: "public".to_string(),
                view_type: rbeaver::database::ViewType::Materialized,
                definition: Some("SELECT p.name, SUM(oi.quantity * oi.price) as total_sales FROM products p JOIN order_items oi ON p.id = oi.product_id GROUP BY p.id, p.name".to_string()),
                comment: Some("Materialized view for product sales analytics".to_string()),
                owner: Some("postgres".to_string()),
                is_updatable: false,
            },
        ];

        self.database_tree
            .set_views(&connection_id, "public".to_string(), sample_views);

        self.test_connection_added = true;
    }
}

impl eframe::App for ObjectCategorizationTestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RBeaver - Database Object Categorization Test");
            ui.separator();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading("Features Demonstrated:");
                    ui.label("✅ Hierarchical tree structure (Database → Schemas → Object Types → Objects)");
                    ui.label("✅ Object counts display (e.g., 'Tables (15)', 'Views (3)')");
                    ui.label("✅ Multiple object categories (Tables, Views, Functions, Triggers, Sequences, Indexes)");
                    ui.label("✅ Search and filtering functionality");
                    ui.label("✅ Context menus with object-specific actions");
                    ui.label("✅ Light theme styling consistency");
                    ui.label("✅ Lazy loading support for performance");
                    ui.separator();
                    ui.label("📝 Instructions:");
                    ui.label("• Expand schemas to see object categories");
                    ui.label("• Click the search icon (🔍) to enable filtering");
                    ui.label("• Right-click on objects for context menus");
                    ui.label("• Object counts are shown next to category names");
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.heading("Database Tree:");
                    egui::ScrollArea::vertical()
                        .max_height(600.0)
                        .show(ui, |ui| {
                            self.database_tree.render(ui);
                        });
                });
            });
        });
    }
}
