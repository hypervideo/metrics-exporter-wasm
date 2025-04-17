use super::{
    Error,
    Events,
    Result,
};
use crate::Event;
use asn1rs::prelude::*;

impl Events {
    /// Serialize the events using asn1.
    pub fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = UperWriter::default();
        writer
            .write(self)
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(writer.into_bytes_vec())
    }

    /// Deserialize from asn1.
    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut reader = UperReader::from(Bits::from(data));
        reader
            .read::<Events>()
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))
    }

    #[cfg(feature = "compress-brotli")]
    /// Serialize the events using asn1 and compress using brotli.
    pub fn encode_and_compress_br(&self) -> Result<Vec<u8>> {
        let encoded = self.encode()?;

        let mut compressed = Vec::new();
        {
            use std::io::Write as _;
            let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, 11, 22);
            writer
                .write_all(&encoded)
                .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
        }

        Ok(compressed)
    }

    /// Serialize the events using asn1 and compress using zstd. This requires [`crate::zstd_external::initialize`] to
    /// be called first!
    #[cfg(feature = "compress-zstd-external")]
    pub fn encode_and_compress_zstd_external(&self, level: u8) -> Result<Vec<u8>> {
        use wasm_bindgen::prelude::*;
        use web_sys::js_sys::Uint8Array;

        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = zstd)]
            fn compress(buf: Uint8Array, level: u32) -> Uint8Array;
        }
        let encoded = self.encode()?;
        let compressed = compress(Uint8Array::from(encoded.as_slice()), level as _);
        Ok(compressed.to_vec())
    }
}

impl From<Events> for Vec<Event> {
    fn from(value: Events) -> Self {
        value.0.into_iter().map(Event::from).collect()
    }
}

impl From<Vec<Event>> for Events {
    fn from(value: Vec<Event>) -> Self {
        Events(value.into_iter().map(Into::into).collect())
    }
}
