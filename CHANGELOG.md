# Changelog

This package tries to adhere to [semver](https://semver.org/).

## [0.2.1]
### Add: Feature `serde` to enable serde `Serialize` and `Deserialize` for metric events

## [0.2.0]
### Add: Track time of events
- Introduces `RecordedEvent` that carries a timestamp. Those get grouped into `RecordedEvents` collections that represent batches of events. They mostly replace `Events` and are now send by `metrics_exporter_wasm::metrics_http_sender::post_metrics`.

## [0.1.0]
### Add: Initial version
