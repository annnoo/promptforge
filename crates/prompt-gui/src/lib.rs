pub mod app;
pub mod modals;
pub mod panels;
pub mod theme;

pub use app::PromptForgeGuiApp;
use prompt_persistence::load_document;
use std::path::PathBuf;

/// Launches the native desktop GUI workbench using egui + eframe.
pub fn run_gui(initial_file: Option<PathBuf>) -> Result<(), eframe::Error> {
    let initial_doc = if let Some(ref path) = initial_file {
        if path.exists() {
            load_document(path).ok()
        } else {
            None
        }
    } else {
        None
    };

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("PromptForge — Prompt Engineering Workbench")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "PromptForge",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(PromptForgeGuiApp::new(initial_doc, initial_file)))
        }),
    )
}
