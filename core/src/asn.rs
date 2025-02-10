use crate::{
    Event,
    MetricOperation,
    MetricType,
};
use asn1rs::prelude::*;
pub use generated::Events;
use metrics::{
    Key,
    Unit,
};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/metrics.rs"));
}

pub type Error = std::io::Error;
pub type Result<T> = std::result::Result<T, Error>;

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

impl From<generated::Event> for Event {
    fn from(value: generated::Event) -> Self {
        use generated::{
            EventDescription,
            EventMetric,
            EventMetricKey,
        };
        match value {
            generated::Event::Description(EventDescription {
                key_name: name,
                metric_type,
                unit,
                description,
            }) => Event::Description {
                name: name.into(),
                metric_type: metric_type.into(),
                unit: unit.map(Into::into),
                description: description.into(),
            },

            generated::Event::Metric(EventMetric {
                key: EventMetricKey { name, label },
                op,
            }) => {
                let labels = label
                    .into_iter()
                    .map(|entry| metrics::Label::new(entry.key, entry.value))
                    .collect::<Vec<_>>();
                let key = Key::from_parts(name, labels);
                Event::Metric { key, op: op.into() }
            }
        }
    }
}

#[rustfmt::skip]
impl From<generated::MetricOperation> for MetricOperation {
    fn from(value: generated::MetricOperation) -> Self {
        use generated::MetricOperation::*;
        match value {
            IncrementCounter(val) => MetricOperation::IncrementCounter(val as _),
            SetCounter(val) => MetricOperation::SetCounter(val as _),
            IncrementGauge(val) => MetricOperation::IncrementGauge(f64::from_be_bytes(val.try_into().unwrap())),
            DecrementGauge(val) => MetricOperation::DecrementGauge(f64::from_be_bytes(val.try_into().unwrap())),
            SetGauge(val) => MetricOperation::SetGauge(f64::from_be_bytes(val.try_into().unwrap())),
            RecordHistogram(val) => MetricOperation::RecordHistogram(f64::from_be_bytes(val.try_into().unwrap())),
        }
    }
}

impl From<generated::MetricType> for MetricType {
    fn from(value: generated::MetricType) -> Self {
        match value {
            generated::MetricType::Counter => MetricType::Counter,
            generated::MetricType::Gauge => MetricType::Gauge,
            generated::MetricType::Histogram => MetricType::Histogram,
        }
    }
}

impl From<generated::Unit> for Unit {
    fn from(value: generated::Unit) -> Self {
        match value {
            generated::Unit::Count => Unit::Count,
            generated::Unit::Percent => Unit::Percent,
            generated::Unit::Seconds => Unit::Seconds,
            generated::Unit::Milliseconds => Unit::Milliseconds,
            generated::Unit::Microseconds => Unit::Microseconds,
            generated::Unit::Nanoseconds => Unit::Nanoseconds,
            generated::Unit::Tebibytes => Unit::Tebibytes,
            generated::Unit::Gibibytes => Unit::Gibibytes,
            generated::Unit::Mebibytes => Unit::Mebibytes,
            generated::Unit::Kibibytes => Unit::Kibibytes,
            generated::Unit::Bytes => Unit::Bytes,
            generated::Unit::TerabitsPerSecond => Unit::TerabitsPerSecond,
            generated::Unit::GigabitsPerSecond => Unit::GigabitsPerSecond,
            generated::Unit::MegabitsPerSecond => Unit::MegabitsPerSecond,
            generated::Unit::KilobitsPerSecond => Unit::KilobitsPerSecond,
            generated::Unit::BitsPerSecond => Unit::BitsPerSecond,
            generated::Unit::CountPerSecond => Unit::CountPerSecond,
        }
    }
}

impl From<Vec<Event>> for Events {
    fn from(value: Vec<Event>) -> Self {
        Events(value.into_iter().map(Into::into).collect())
    }
}

impl From<Event> for generated::Event {
    fn from(value: Event) -> Self {
        use generated::{
            EventDescription,
            EventMetric,
            EventMetricKey,
            EventMetricKeyLabel,
        };
        match value {
            Event::Description {
                name,
                metric_type,
                unit,
                description,
            } => generated::Event::Description(EventDescription {
                key_name: name.as_str().into(),
                metric_type: metric_type.into(),
                unit: unit.map(Into::into),
                description: description.to_string(),
            }),

            Event::Metric { key, op } => {
                let (key_name, key_labels) = key.into_parts();

                generated::Event::Metric(EventMetric {
                    key: EventMetricKey {
                        name: key_name.as_str().into(),
                        label: key_labels
                            .into_iter()
                            .map(|label| {
                                let (key, value) = label.into_parts();
                                EventMetricKeyLabel {
                                    key: key.to_string(),
                                    value: value.to_string(),
                                }
                            })
                            .collect(),
                    },
                    op: op.into(),
                })
            }
        }
    }
}

#[rustfmt::skip]
impl From<MetricOperation> for generated::MetricOperation {
    fn from(value: MetricOperation) -> Self {
        use generated::MetricOperation::*;
        match value {
            MetricOperation::IncrementCounter(val) => IncrementCounter(val),
            MetricOperation::SetCounter(val) => SetCounter(val),
            MetricOperation::IncrementGauge(val) => IncrementGauge(val.to_be_bytes().to_vec()),
            MetricOperation::DecrementGauge(val) => DecrementGauge(val.to_be_bytes().to_vec()),
            MetricOperation::SetGauge(val) => SetGauge(val.to_be_bytes().to_vec()),
            MetricOperation::RecordHistogram(val) => RecordHistogram(val.to_be_bytes().to_vec()),
        }
    }
}

impl From<MetricType> for generated::MetricType {
    fn from(value: MetricType) -> Self {
        match value {
            MetricType::Counter => generated::MetricType::Counter,
            MetricType::Gauge => generated::MetricType::Gauge,
            MetricType::Histogram => generated::MetricType::Histogram,
        }
    }
}

impl From<Unit> for generated::Unit {
    fn from(value: Unit) -> Self {
        match value {
            Unit::Count => generated::Unit::Count,
            Unit::Percent => generated::Unit::Percent,
            Unit::Seconds => generated::Unit::Seconds,
            Unit::Milliseconds => generated::Unit::Milliseconds,
            Unit::Microseconds => generated::Unit::Microseconds,
            Unit::Nanoseconds => generated::Unit::Nanoseconds,
            Unit::Tebibytes => generated::Unit::Tebibytes,
            Unit::Gibibytes => generated::Unit::Gibibytes,
            Unit::Mebibytes => generated::Unit::Mebibytes,
            Unit::Kibibytes => generated::Unit::Kibibytes,
            Unit::Bytes => generated::Unit::Bytes,
            Unit::TerabitsPerSecond => generated::Unit::TerabitsPerSecond,
            Unit::GigabitsPerSecond => generated::Unit::GigabitsPerSecond,
            Unit::MegabitsPerSecond => generated::Unit::MegabitsPerSecond,
            Unit::KilobitsPerSecond => generated::Unit::KilobitsPerSecond,
            Unit::BitsPerSecond => generated::Unit::BitsPerSecond,
            Unit::CountPerSecond => generated::Unit::CountPerSecond,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn metric_serialization() {
        let event = Event::Metric {
            key: Key::from_parts("some-key", &[("key", "value")]),
            op: MetricOperation::SetGauge(42.2312313213f64),
        };
        let events = Events::from(vec![event]);

        let bytes = events.encode().unwrap();
        assert_eq!(bytes.len(), 30);

        let events2 = Events::decode(&bytes).unwrap();
        assert_eq!(events, events2);
    }
}
