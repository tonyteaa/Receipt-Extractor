mod app;
mod database;
mod keygen;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("Receipt Extractor - Admin Tool"),
        ..Default::default()
    };

    eframe::run_native(
        "Receipt Extractor Admin",
        options,
        Box::new(|cc| Ok(Box::new(app::AdminApp::new(cc)))),
    )
}

