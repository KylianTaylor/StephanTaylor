// ──────────────────────────────────────────────────────────────────────────────
// Nimbuzyn — Android native entry point
// ──────────────────────────────────────────────────────────────────────────────

#![cfg_attr(not(debug_assertions), deny(warnings))]
#![allow(clippy::new_without_default)]

pub mod app;
pub mod db;
pub mod models;
pub mod screens;
pub mod theme;

use crate::app::NimbuzynApp;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: android_activity::AndroidApp) {
    use android_activity::AndroidApp;

    // Initialize Android logger
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("Nimbuzyn"),
    );

    log::info!("Nimbuzyn iniciando…");

    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    eframe::run_native(
        "Nimbuzyn",
        options,
        Box::new(|cc| Box::new(NimbuzynApp::new(cc))),
    )
    .expect("Error al iniciar eframe");
}
