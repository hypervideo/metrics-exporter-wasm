use super::log;
use metrics_exporter_wasm::WasmRecorder;
use metrics_util::layers::FanoutBuilder;
use std::sync::OnceLock;
use wasm_bindgen::prelude::*;

static SNAPSHOTTER: OnceLock<metrics_util::debugging::Snapshotter> = OnceLock::new();

#[wasm_bindgen(js_name = setup_metrics_test)]
pub fn setup() {
    let debugging_recorder = metrics_util::debugging::DebuggingRecorder::new();
    let snapshotter = debugging_recorder.snapshotter();
    let _ = SNAPSHOTTER.set(snapshotter);

    let wasm_recorder = WasmRecorder::builder().build().expect("failed to install recorder");

    let recorder = FanoutBuilder::default()
        .add_recorder(debugging_recorder)
        .add_recorder(wasm_recorder)
        .build();

    metrics::set_global_recorder(recorder).expect("failed to set global recorder");

    log!("metrics setup complete");
}

#[wasm_bindgen(js_name = run_metrics_test)]
pub fn run() {
    for _ in 0..10 {
        do_something();
    }

    let snapshotter = SNAPSHOTTER.get().expect("snapshotter not set");
    let snapshot = snapshotter.snapshot().into_vec();

    log!("metrics test complete {snapshot:?}");
}

pub fn do_something() {
    // let labels = [("i", format!("{}!", i))];
    // metrics::counter!("invocations", &labels).increment(1);

    metrics::counter!("invocations").increment(1);
}
