mod app;
mod debug_log;
mod document_processor;
mod ai_extractor;
mod csv_exporter;
mod ocr;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([600.0, 400.0])
            .with_decorations(true)  // Enable window decorations (title bar with buttons)
            .with_resizable(true)    // Allow window resizing
            .with_maximize_button(true)  // Enable maximize button
            .with_minimize_button(true)  // Enable minimize button
            .with_close_button(true)     // Enable close button
            .with_drag_and_drop(true),   // Enable drag and drop from OS file manager
        persist_window: true,  // Persist window size and position
        ..Default::default()
    };

    eframe::run_native(
        "Receipt Data Extractor",
        options,
        Box::new(|cc| Ok(Box::new(app::ReceiptExtractorApp::new(cc)))),
    )
}

