use metrics_exporter_wasm::WasmRecorder;
use std::time::Duration;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn setup(endpoint: &str) {
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

    let recorder = WasmRecorder::builder()
        .endpoint(endpoint.to_string())
        .send_frequency(Duration::from_secs(5))
        .build()
        .expect("failed to create recorder");

    metrics::set_global_recorder(recorder).expect("failed to set global recorder");

    tracing::info!("setup complete");
}

#[wasm_bindgen]
pub async fn run() {
    for i in 0..10 {
        do_something(i).await;
    }

    tracing::info!("metrics test complete");
}

pub async fn do_something(i: i64) {
    let labels = [("i", format!("{}!", i))];
    metrics::counter!("invocations", &labels).increment(1);

    gloo::timers::future::sleep(std::time::Duration::from_millis(100)).await;
}
