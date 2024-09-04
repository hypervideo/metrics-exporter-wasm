use metrics::{Key, KeyName, SharedString, Unit};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum MetricOperation {
    IncrementCounter(u64),
    SetCounter(u64),
    IncrementGauge(f64),
    DecrementGauge(f64),
    SetGauge(f64),
    RecordHistogram(f64),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Metadata {
        name: KeyName,
        metric_type: MetricType,
        unit: Option<Unit>,
        description: SharedString,
    },
    Metric {
        key: Key,
        op: MetricOperation,
    },
}
