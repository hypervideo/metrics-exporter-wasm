default:
    just --list

# -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

build:
    cargo build --release --target wasm32-unknown-unknown
    wasm-bindgen --target web --out-dir dist ./target/wasm32-unknown-unknown/release/metrics_exporter_wasm.wasm
    wasm-opt -O4 ./dist/metrics_exporter_wasm_bg.wasm -o ./dist/metrics_exporter_wasm_bg.optimized.wasm

build-debug:
    cargo build --target wasm32-unknown-unknown

print-wasm-size: build
    du -b ./dist/*.wasm | numfmt --to=iec-i --format="%3.5f"

# -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

example-1-build:
    cd examples/standalone-client && just build

example-1-serve:
    cd examples/standalone-client && just dev

example-1-serve-release:
    cd examples/standalone-client && just serve

example-2-serve:
    cd examples/server-and-client && just serve

# -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

release *args="":
    cargo rdme
    cargo release {{ args }}
