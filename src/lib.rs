mod asn_benchmark;
mod benchmark;
mod recorder;
pub mod types;

use wasm_bindgen::prelude::*;

#[macro_use]
extern crate tracing;

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
    info!("setup");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn run_asn_benchmark() {
    asn_benchmark::run();
}
