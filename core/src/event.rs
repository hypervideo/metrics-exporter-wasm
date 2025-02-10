use metrics::{
    Key,
    KeyName,
    SharedString,
    Unit,
};

// These types are "public" interface. The asn1 generated types are a bit more
// complex, to simplify, we provide these representations that can be converted
// back and forth.

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Description {
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

/// The metric type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// A counter is a cumulative metric that represents a single monotonically
    /// increasing counter whose value can only increase or be reset to zero on
    /// restart. For example, you can use a counter to represent the number of
    /// requests served, tasks completed, or errors.
    Counter,
    /// A gauge is a metric that represents a single numerical value that can
    /// arbitrarily go up and down. Gauges are typically used for measured
    /// values like temperatures or current memory usage, but also "counts" that
    /// can go up and down, like the number of concurrent requests.
    Gauge,
    /// A histogram samples observations (usually things like request durations
    /// or response sizes) and counts them in configurable buckets. It also
    /// provides a sum of all observed values.
    Histogram,
}

/// Describes what the metric operation does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricOperation {
    /// Increment a counter by a given value.
    IncrementCounter(u64),
    /// Set a counter to a given value.
    SetCounter(u64),
    /// Increment a gauge by a given value.
    IncrementGauge(f64),
    /// Decrement a gauge by a given value.
    DecrementGauge(f64),
    /// Set a gauge to a given value.
    SetGauge(f64),
    /// Record a histogram value.
    RecordHistogram(f64),
}
