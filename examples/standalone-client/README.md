# metrics-wasm-exporter client-standalone example

This shows how to setup the metrics-wasm-exporter crate and receive metrics from it using a `tokio::sync::broadcast` channel. Depending on the use case, it might be simpler to just implement [`metrics::Recorder`](https://docs.rs/metrics/latest/metrics/trait.Recorder.html).
