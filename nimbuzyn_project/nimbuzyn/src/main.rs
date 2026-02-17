// Desktop runner for development/testing on PC
// Build with: cargo run --bin nimbuzyn
// For Android: cargo apk build --lib

#[cfg(not(target_os = "android"))]
fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Nimbuzyn")
            .with_inner_size([390.0, 844.0])   // iPhone 14 Pro resolution ratio
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Nimbuzyn",
        options,
        Box::new(|cc| Box::new(nimbuzyn::app::NimbuzynApp::new(cc))),
    )
    .expect("Error al iniciar Nimbuzyn");
}

#[cfg(target_os = "android")]
fn main() {
    // Entry point on Android is `android_main` in lib.rs
    unreachable!("Use android_main on Android");
}
