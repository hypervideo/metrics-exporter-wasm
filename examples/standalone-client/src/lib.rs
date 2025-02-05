mod asn_benchmark;
mod benchmark;
mod metrics_test;

pub use metrics_test::{
    run as run_metrics_test,
    setup as setup_metrics_test,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn setup() {
    console_error_panic_hook::set_once();

    // Register tracing subscriber.
    {
        let mut config = wasm_tracing::WasmLayerConfig::new();
        config.set_max_level(tracing::Level::TRACE);
        #[cfg(debug_assertions)]
        if let Some(origin_base_url) = option_env!("WASM_TRACING_BASE_URL") {
            config.set_origin_base_url(origin_base_url);
        }
        wasm_tracing::set_as_global_default_with_config(config).expect("Failed to set as global default");
    }

    tracing::info!("tracing setup complete");
}

#[wasm_bindgen]
pub fn run_asn_benchmark() {
    asn_benchmark::run();
}
