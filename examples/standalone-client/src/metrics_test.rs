use metrics_exporter_wasm::WasmRecorder;
use tokio::sync::broadcast::error::RecvError;
use wasm_bindgen::prelude::*;

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
