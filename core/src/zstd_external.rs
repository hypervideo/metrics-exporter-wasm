use wasm_bindgen::prelude::*;
use web_sys::js_sys::{
    eval,
    ArrayBuffer,
    Uint8Array,
};

/// This loads the bundled [zstd-wasm](https://github.com/bokuweb/zstd-wasm) library.
pub async fn initialize() {
    #[cfg(feature = "compress-zstd-external-from-source")]
    const SOURCE: &str = include_str!(concat!(env!("OUT_DIR"), "/zstd-bundle.js"));
    #[cfg(not(feature = "compress-zstd-external-from-source"))]
    const SOURCE: &str = include_str!("../zstd-bundle.js");
    eval(SOURCE).expect("failed to load zstd");

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = zstd)]
        async fn init(buf: ArrayBuffer) -> JsValue;
    }

    #[cfg(feature = "compress-zstd-external-from-source")]
    const RAW_WASM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/zstd.wasm"));
    #[cfg(not(feature = "compress-zstd-external-from-source"))]
    const RAW_WASM: &[u8] = include_bytes!("../zstd.wasm");
    let array = Uint8Array::from(RAW_WASM);
    let buf = array.buffer();
    init(buf).await;
    tracing::debug!("zstd intialized");
}
