mod asn_benchmark;
mod benchmark;
mod metrics_test;

pub use metrics_test::run as run_metrics_test;

use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => (gloo::console::log!(&format_args!($($t)*).to_string()))
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (println!($($t)*))
}

#[wasm_bindgen]
pub fn setup() {
    color_eyre::install().expect("color_eyre init");
    console_error_panic_hook::set_once();
    wasm_tracing::set_as_global_default();
    log!("tracing setup complete");
}

#[wasm_bindgen]
pub fn run_asn_benchmark() {
    asn_benchmark::run();
}
