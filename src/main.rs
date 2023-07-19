#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod types;
pub use app::MoekkiCalcApp;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "moekki-calc-native",
        native_options,
        Box::new(|cc| Box::new(MoekkiCalcApp::new(cc))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "moekki-calc-wasm",
        web_options,
        Box::new(|cc| Box::new(MoekkiCalcApp::new(cc))),
    )
    .expect("failed to start wasm eframe");
}
