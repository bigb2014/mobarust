//! moba-gui -- MobaRust application shell.
//!
//! Tabbed terminal + multi-protocol remote client GUI.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use eframe::egui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MobaRust")
            .with_inner_size([800.0, 600.0])
            .with_position(egui::Pos2::new(100.0, 100.0))
            .with_visible(true)
            .with_active(true),
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    let app = match moba_gui::MobaApp::new(24, 80) {
        Ok(app) => app,
        Err(e) => {
            tracing::error!("PTY spawn failed: {e}");
            moba_gui::MobaApp::new_empty(24, 80)
        }
    };

    eframe::run_native("MobaRust", options, Box::new(move |_cc| Ok(Box::new(app))))?;
    Ok(())
}