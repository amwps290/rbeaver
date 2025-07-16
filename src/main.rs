mod app;
mod config;
mod database;
mod ui;
mod utils;

use app::RBeaverApp;
use ui::{setup_chinese_fonts, setup_light_theme};
use utils::init_logging;

fn main() -> Result<(), eframe::Error> {
    // Initialize logging
    init_logging();

    log::info!("Starting RBeaver Database Management Tool");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("RBeaver - Database Management Tool"),
        ..Default::default()
    };

    eframe::run_native(
        "RBeaver",
        options,
        Box::new(|cc| {
            // Setup Chinese font support
            if let Err(e) = setup_chinese_fonts(&cc.egui_ctx) {
                log::warn!("Failed to setup Chinese fonts: {}", e);
                log::info!("Continuing with default Unicode font support");
            }

            // Setup light theme
            setup_light_theme(&cc.egui_ctx);

            // Create and return the app
            Ok(Box::new(RBeaverApp::new(cc)))
        }),
    )
}
