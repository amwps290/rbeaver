use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

/// Initialize logging for the application
pub fn init_logging() {
    let mut builder = Builder::from_default_env();

    builder
        .target(Target::Stdout)
        .filter_level(LevelFilter::Info)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] [{}:{}] {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();

    log::info!("Logging initialized");
}

/// Log database operations
pub fn log_query(sql: &str, execution_time: std::time::Duration) {
    log::info!(
        "Query executed in {:.2}ms: {}",
        execution_time.as_millis(),
        sql.chars().take(100).collect::<String>()
    );
}

/// Log connection events
pub fn log_connection_event(event: &str, connection_info: &str) {
    log::info!("Connection {}: {}", event, connection_info);
}

/// Log errors with context
pub fn log_error(context: &str, error: &dyn std::error::Error) {
    log::error!("{}: {}", context, error);
}
