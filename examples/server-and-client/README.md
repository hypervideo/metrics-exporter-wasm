# metrics-wasm-exporter client/server example

This shows how to setup the metrics-wasm-exporter and an axum server to continously send metrics to the server. The metrics data is encoded using [ASN.1](https://github.com/kellerkindt/asn1rs) and compressed using brotli.
