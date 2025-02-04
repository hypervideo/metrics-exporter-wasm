use super::log;
use metrics_exporter_wasm::WasmRecorder;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = setup_metrics_test)]
pub fn setup() {
    // let recorder = metrics_util::debugging::DebuggingRecorder::new();
    // let snapshotter = recorder.snapshotter();
    // let snap = snapshotter.snapshot();

    let recorder = WasmRecorder::builder().build().expect("failed to install recorder");

    recorder.install().expect("failed to install recorder");

    log!("metrics setup complete");
}

#[wasm_bindgen(js_name = run_metrics_test)]
pub fn run() {
    for _ in 0..10 {
        do_something();
    }
    log!("metrics test complete");
}

pub fn do_something() {
    // let labels = [("i", format!("{}!", i))];
    // metrics::counter!("invocations", &labels).increment(1);

    metrics::counter!("invocations").increment(1);
}
