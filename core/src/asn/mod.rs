mod event;
mod events;
mod metric_operation;
mod metric_type;
mod unit;

pub use generated::Events;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/metrics.rs"));
}

pub type Error = std::io::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {

    use crate::{
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
