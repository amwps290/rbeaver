pub mod components;
pub mod confirmation_dialog;
pub mod connection_dialog;
pub mod database_tree;
pub mod query_editor;
pub mod result_table;
pub mod theme;

// Re-export main UI components
pub use confirmation_dialog::ConfirmationDialog;
pub use connection_dialog::{ConnectionDialog, DialogAction};
pub use database_tree::{ConnectionAction, DatabaseTree, TreeItem};
pub use query_editor::QueryEditor;
pub use result_table::ResultTable;
pub use theme::setup_light_theme;
