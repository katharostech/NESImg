#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    setup_tracing();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "NESImg",
        native_options,
        Box::new(|cc| Box::new(nesimg_gui::NesimgGui::new(cc))),
    );
}

fn setup_tracing() {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer();
    let filter_layer =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("nesimg_gui=debug"));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();
}
