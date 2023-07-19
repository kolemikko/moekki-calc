#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod types;
pub use app::MoekkiCalcApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "moekki-calc-native",
        native_options,
        Box::new(|cc| Box::new(MoekkiCalcApp::new(cc))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "moekki-calc-wasm",
                web_options,
                Box::new(|cc| Box::new(MoekkiCalcApp::new(cc))),
            )
            .await
            .expect("failed to start wasm eframe");
    });
}
