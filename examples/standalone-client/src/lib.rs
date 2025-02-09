mod benchmarks;
mod util;

use metrics_exporter_wasm::WasmRecorder;
use tokio::sync::broadcast::error::RecvError;
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

#[wasm_bindgen(js_name = setup_metrics_test)]
pub fn setup_metrics_test() {
    let recorder = WasmRecorder::builder()
        .buffer_size(25)
        .build()
        .expect("failed to install recorder");

    let mut rx = recorder.subscribe();

    metrics::set_global_recorder(recorder).expect("failed to set global recorder");

    wasm_bindgen_futures::spawn_local(async move {
        tracing::debug!("metrics stream started");
        loop {
            match rx.recv().await {
                Ok(event) => {
                    tracing::info!(?event, "metrics");
                }
                Err(RecvError::Closed) => {
                    break;
                }
                Err(RecvError::Lagged(lag)) => {
                    tracing::warn!(?lag, "metrics lagging");
                }
            }
        }
        tracing::debug!("metrics stream closed");
    });

    tracing::debug!("metrics setup complete");
}

#[wasm_bindgen(js_name = run_metrics_test)]
pub fn run_metrics_test() {
    for _ in 0..10 {
        do_something();
    }
}

pub fn do_something() {
    metrics::counter!("test.foo").increment(1);
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

#[wasm_bindgen]
pub fn run_benchmarks() {
    benchmarks::run();
}
