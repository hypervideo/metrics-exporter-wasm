default:
    just --list

build-wasm:
    cargo build --release --target wasm32-unknown-unknown
    wasm-bindgen --target web --out-dir dist ./target/wasm32-unknown-unknown/release/metrics_exporter_wasm.wasm

build-wasm-debug:
    rm -rf ./dist
    CARGOcargo build --target wasm32-unknown-unknown
    wasm-bindgen --target web --out-dir dist ./target/wasm32-unknown-unknown/debug/metrics_exporter_wasm.wasm

serve-live: build-wasm
    concurrently --kill-others \
      "fd -e rs | entr just build-wasm" \
      "live-server --no-browser"

serve: build-wasm
    concurrently --kill-others \
      "fd -e rs | entr just build-wasm" \
      "http-server"

NAME := "asn1-eval"

print-wasm-size: build-wasm
    du -b ./target/wasm32-unknown-unknown/release/*.wasm | numfmt --to=iec-i --format="%3.5f"
