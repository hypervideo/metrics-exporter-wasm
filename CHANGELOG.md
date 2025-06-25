# Changelog

This package tries to adhere to [semver](https://semver.org/).

## [0.4.0]
### Add/Change:
Rename `metrics_exporter_wasm::MetricsHttpSender::start_with` to `metrics_exporter_wasm::MetricsHttpSender::start_with_metrics_recorder` and add `metrics_exporter_wasm::metrics_http_sender::MetricsHttpSender::start_with_metrics_recorder_and_filter` to allow to filter events which should be sent to the server.

## [0.3.0]
### Add/Change:
This introduces a Transport trait that on the one hand allows to run the wasm
metrics exporter with a different Transport implementation. It also makes the
Transport trait and its (currently) sole implementation HttpPostTransport
available as part of the metrics-exporter-wasm API. Since the Transport trait
allows to send a bytes::Bytes payload, the transport can now be used for sending
non-metrics data as well. This can be handy if the metrics ingestion pipeline
should accept data other than metrics metrics.

## [0.2.1]
### Add: Feature `serde` to enable serde `Serialize` and `Deserialize` for metric events

## [0.2.0]
### Add: Track time of events
- Introduces `RecordedEvent` that carries a timestamp. Those get grouped into `RecordedEvents` collections that represent batches of events. They mostly replace `Events` and are now send by `metrics_exporter_wasm::metrics_http_sender::post_metrics`.

## [0.1.0]
### Add: Initial version
