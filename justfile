default:
    just --list

# -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

build:
    rm -rf ./dist
    rm -rf ./target/wasm32-unknown-unknown/release/*.wasm
    cargo build --release --target wasm32-unknown-unknown
    wasm-bindgen --target web --out-dir dist ./target/wasm32-unknown-unknown/release/metrics_exporter_wasm.wasm
    wasm-opt -O4 ./dist/metrics_exporter_wasm_bg.wasm -o ./dist/metrics_exporter_wasm_bg.optimized.wasm

build-release-with-debug:
    rm -rf ./dist
    rm -rf ./target/wasm32-unknown-unknown/release-with-debug/*.wasm
    cargo build --profile release-with-debug --target wasm32-unknown-unknown
    wasm-bindgen --target web --keep-debug --out-dir dist ./target/wasm32-unknown-unknown/release-with-debug/metrics_exporter_wasm.wasm

build-debug:
    cargo build --target wasm32-unknown-unknown

print-wasm-size: build
    du -b ./dist/*.wasm | numfmt --to=iec-i --format="%3.5f"
    du -b ./target/wasm32-unknown-unknown/release/*.wasm | numfmt --to=iec-i --format="%3.5f"
    cd examples/server-and-client/client && just print-wasm-size

check:
    cargo clippy -p metrics-exporter-wasm --target wasm32-unknown-unknown -- -D warnings
    cargo clippy -p metrics-exporter-wasm --features compress-zstd-external --target wasm32-unknown-unknown -- -D warnings
    cargo clippy -p metrics-exporter-wasm --features compress-brotli --target wasm32-unknown-unknown -- -D warnings
    cargo clippy -p metrics-exporter-wasm-core --features serde --target wasm32-unknown-unknown -- -D warnings

test:
    cargo nextest run --workspace

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
