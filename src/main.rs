mod app;
mod config;
mod database;
mod ui;
mod utils;

use app::RBeaverApp;
use egui_chinese_font::setup_chinese_fonts;
use ui::setup_light_theme;
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
            // load chinese font
            // setup_chinese_fonts(&cc.egui_ctx).expect("无法加载中文字体");

            // Setup light theme
            setup_light_theme(&cc.egui_ctx);

            // Create and return the app
            Ok(Box::new(RBeaverApp::new(cc)))
        }),
    )
}
