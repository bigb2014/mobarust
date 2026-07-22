//! moba-gui -- MobaRust application shell.
//!
//! Tabbed terminal + multi-protocol remote client GUI.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use eframe::egui;

fn main() -> eframe::Result<()> {
    // Set up tracing.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MobaRust")
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "MobaRust",
        options,
        Box::new(|_cc| {
            Ok(Box::new(moba_gui::MobaApp::new(24, 80).map_err(|e| {
                eframe::Error::AppCreation(Box::new(std::io::Error::other(e)))
            })?))
        }),
    )
}
