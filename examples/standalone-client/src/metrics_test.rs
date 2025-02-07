use metrics_exporter_wasm::WasmRecorder;
use metrics_util::layers::FanoutBuilder;
use std::sync::OnceLock;
use tokio::sync::broadcast::error::RecvError;
use wasm_bindgen::prelude::*;

static SNAPSHOTTER: OnceLock<metrics_util::debugging::Snapshotter> = OnceLock::new();

#[wasm_bindgen(js_name = setup_metrics_test)]
pub fn setup() {
    let debugging_recorder = metrics_util::debugging::DebuggingRecorder::new();
    let snapshotter = debugging_recorder.snapshotter();
    let _ = SNAPSHOTTER.set(snapshotter);

    let wasm_recorder = WasmRecorder::builder()
        .buffer_size(25)
        .build()
        .expect("failed to install recorder");

    let mut rx = wasm_recorder.subscribe();

    let recorder = FanoutBuilder::default()
        .add_recorder(debugging_recorder)
        .add_recorder(wasm_recorder)
        .build();

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
pub fn run() {
    for _ in 0..10 {
        do_something();
    }

    let snapshotter = SNAPSHOTTER.get().expect("snapshotter not set");
    let snapshot = snapshotter.snapshot().into_vec();

    tracing::debug!("metrics test complete {snapshot:?}");
}

pub fn do_something() {
    metrics::counter!("test.foo").increment(1);
}
