<!-- cargo-rdme start -->

# metrics-wasm-exporter

This is a WASM implementation of a [metrics](https://github.com/metrics-rs/metrics) [Recorder](https://docs.rs/metrics/latest/metrics/trait.Recorder.html).

Metrics can be transferred in two ways:

- If the WASM app itself wants access to metrics, it can register an event receiver that will be called whenever a
  metric is recorded.
- Metrics can be send to a remote server. In this case, [asn1](https://github.com/kellerkindt/asn1rs) is used to
  encode the metrics into a space efficient binary format. The encoded metrics are then batched and send with POST
  requests to the specified server URL.

<!-- cargo-rdme end -->
