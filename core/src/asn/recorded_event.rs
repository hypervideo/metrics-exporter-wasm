use super::{
    generated,
    Asn1Decode,
    Asn1Encode,
    Error,
    Result,
};
use crate::{
    util_time,
    Event,
    RecordedEvent,
};
use asn1rs::prelude::*;
use chrono::prelude::*;

impl RecordedEvent {
    fn from_asn_with_base_time(event: generated::RecordedEvent, base_time: DateTime<Utc>) -> Self {
        let generated::RecordedEvent { offset_ms, event } = event;
        Self {
            timestamp: base_time + chrono::Duration::milliseconds(offset_ms as i64),
            event: event.into(),
        }
    }

    fn into_asn_with_base_time(self, base_time: DateTime<Utc>) -> generated::RecordedEvent {
        let offset_ms = (self.timestamp - base_time).num_milliseconds() as u32;
        generated::RecordedEvent {
            offset_ms,
            event: self.event.into(),
        }
    }
}

impl From<Event> for RecordedEvent {
    fn from(event: Event) -> Self {
        RecordedEvent {
            timestamp: util_time::now(),
            event,
        }
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

impl generated::RecordedEvents {
    pub fn new(batch_start_time: DateTime<Utc>, events: Vec<RecordedEvent>) -> Self {
        let unix_epoch = Utc.timestamp_opt(0, 0).unwrap();
        let duration = batch_start_time
            .signed_duration_since(unix_epoch)
            .to_std()
            .unwrap_or_default();
        let recording_started_at = generated::Timestamp {
            seconds: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        };

        Self {
            recording_started_at,
            events: events
                .into_iter()
                .map(|event| event.into_asn_with_base_time(batch_start_time))
                .collect(),
        }
    }
}

impl Asn1Encode for generated::RecordedEvents {
    /// Serialize the events using asn1.
    fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = UperWriter::default();
        writer
            .write(self)
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(writer.into_bytes_vec())
    }

    #[cfg(feature = "compress-brotli")]
    /// Serialize the events using asn1 and compress using brotli.
    fn encode_and_compress_br(&self) -> Result<Vec<u8>> {
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
    fn encode_and_compress_zstd_external(&self, level: u8) -> Result<Vec<u8>> {
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

impl Asn1Decode for generated::RecordedEvents {
    /// Deserialize from asn1.
    fn decode(data: &[u8]) -> Result<Self> {
        let mut reader = UperReader::from(Bits::from(data));
        reader
            .read::<generated::RecordedEvents>()
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

impl From<generated::RecordedEvents> for Vec<RecordedEvent> {
    fn from(value: generated::RecordedEvents) -> Self {
        let generated::RecordedEvents {
            recording_started_at,
            events,
        } = value;

        let duration = chrono::Duration::seconds(recording_started_at.seconds as i64)
            + chrono::Duration::nanoseconds(recording_started_at.nanos as i64);
        let unix_epoch = Utc.timestamp_opt(0, 0).unwrap();
        let recording_started_at = unix_epoch + duration;

        events
            .into_iter()
            .map(|event| RecordedEvent::from_asn_with_base_time(event, recording_started_at))
            .collect()
    }
}

impl From<Vec<Event>> for generated::RecordedEvents {
    fn from(events: Vec<Event>) -> Self {
        let now = util_time::now();
        let events = events
            .into_iter()
            .map(|event| RecordedEvent { event, timestamp: now })
            .collect();
        Self::new(now, events)
    }
}
