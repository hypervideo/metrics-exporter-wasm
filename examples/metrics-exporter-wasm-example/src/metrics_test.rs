use super::log;
use metrics_exporter_wasm::WasmRecorder;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = run_metrics_test)]
pub fn run() {
    log!("running metrics test");

    // let recorder = metrics_util::debugging::DebuggingRecorder::new();
    // let _snapshotter = recorder.snapshotter();

    let recorder = WasmRecorder::builder()
        .build()
        .expect("failed to install recorder");

    log!("running metrics test");

    // recorder.install().expect("failed to install recorder");
    // log!("metrics setup complete");

    // for _ in 0..10 {
    //     do_something();
    // }
    // log!("metrics test complete");

    // let snap = snapshotter.snapshot();
    // log!("metrics snapshot: {:?}", snap.into_vec());
}

pub fn do_something() {
    // let labels = [("i", format!("{}!", i))];
    // metrics::counter!("invocations", &labels).increment(1);

    metrics::counter!("invocations").increment(1);
}
