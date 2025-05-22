#[derive(Debug, Clone, Copy)]
pub enum Compression {
    #[cfg(feature = "compress-zstd-external")]
    /// Compress using zstd. This requires [`crate::zstd_external::initialize`] to be called first!
    Zstd { level: u8 },

    #[cfg(feature = "compress-brotli")]
    /// Compress using brotli.
    Brotli,
}

impl Compression {
    #[cfg(feature = "compress-brotli")]
    pub fn compress_br(payload: &bytes::Bytes) -> std::io::Result<bytes::Bytes> {
        let mut compressed = Vec::new();
        {
            use std::io::Write as _;
            let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, 11, 22);
            writer
                .write_all(payload)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        }

        Ok(bytes::Bytes::from(compressed))
    }

    #[cfg(feature = "compress-zstd-external")]
    pub fn compress_zstd_external(payload: &bytes::Bytes, level: u8) -> std::io::Result<bytes::Bytes> {
        use wasm_bindgen::prelude::*;
        use web_sys::js_sys::Uint8Array;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = zstd)]
            fn compress(buf: Uint8Array, level: u32) -> Uint8Array;
        }

        let compressed = compress(Uint8Array::from(payload.as_ref()), level as _);
        Ok(bytes::Bytes::from(compressed.to_vec()))
    }
}
