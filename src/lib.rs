/*!

# metrics-wasm-exporter

[![CI](https://github.com/hypervideo/metrics-exporter-wasm/actions/workflows/ci.yml/badge.svg)](https://github.com/hypervideo/metrics-exporter-wasm/actions/workflows/ci.yml)

This is a WASM implementation of a [metrics](https://github.com/metrics-rs/metrics) [Recorder](https://docs.rs/metrics/latest/metrics/trait.Recorder.html).

Metrics can be transferred in two ways:

- If the WASM app itself wants access to metrics, it can register an event receiver that will be called whenever a
  metric is recorded.
- Metrics can be send to a remote server. In this case, [asn1](https://github.com/kellerkindt/asn1rs) is used to
  encode the metrics into a space efficient binary format. The encoded metrics are then batched and send with POST
  requests to the specified server URL.

Unlike normal metrics, the metrics that metrics-wasm-exporter exports also carry a timestamp of when the metric was
originally recorded.

Example:

```rust
use metrics_exporter_wasm::{WasmRecorder, MetricsHttpSender, HttpPostTransport};
use std::time::Duration;

# fn main() {
let recorder = WasmRecorder::builder()
    .buffer_size(5)
    .build()
    .expect("failed to create recorder");

// Send metrics in regular intervals to a server using HTTP POST requests.
// Will backoff and retry as needed.
const ENDPOINT: &str = "/receive-metrics";
let guard = MetricsHttpSender::new(HttpPostTransport::new().endpoint(ENDPOINT))
    .send_frequency(Duration::from_secs(1))
    .start_with(&recorder);

// Run forever
guard.disarm();

metrics::set_global_recorder(recorder).expect("failed to set global recorder");
# }
```

For how to use compression and how to implement a receiving server handler see the examples at [hypervideo/metrics-exporter-wasm](https://github.com/hypervideo/metrics-exporter-wasm/tree/main/examples/server-and-client).

*/

mod http_transport;
mod metrics_http_sender;
mod recorder;

pub use http_transport::{
    Compression,
    EndpointDefined,
    EndpointUndefined,
    HttpPostTransport,
    Transport,
};
#[cfg(feature = "compress-zstd-external")]
pub use metrics_exporter_wasm_core::zstd_external;
pub use metrics_exporter_wasm_core::{
    Asn1Decode,
    Asn1Encode,
    Event,
    Events,
    MetricOperation,
    MetricType,
    RecordedEvent,
    RecordedEvents,
};
pub use metrics_http_sender::{
    Batch,
    MetricsHttpSender,
};
pub use recorder::{
    WasmRecorder,
    WasmRecorderBuilder,
};

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate scopeguard;
