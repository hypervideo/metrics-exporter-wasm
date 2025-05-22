mod event;
mod events;
mod metric_operation;
mod metric_type;
mod recorded_event;
mod unit;

pub use generated::{
    Events,
    RecordedEvents,
};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/metrics.rs"));
}

pub type Error = std::io::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub trait Asn1Encode {
    fn encode(&self) -> Result<Vec<u8>>;
}

pub trait Asn1Decode: Sized {
    fn decode(data: &[u8]) -> Result<Self>;
}

#[cfg(test)]
mod tests {
    use crate::{
        Asn1Decode,
        Asn1Encode,
        Event,
        Events,
        MetricOperation,
    };
    use metrics::Key;

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
