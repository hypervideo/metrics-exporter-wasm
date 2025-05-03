use chrono::prelude::*;
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa-schema", derive(utoipa::ToSchema))]
pub struct RecordedEvent {
    pub timestamp: DateTime<Utc>,
    pub event: Event,
}

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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa-schema", derive(utoipa::ToSchema))]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "utoipa-schema", derive(utoipa::ToSchema))]
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

#[cfg(any(feature = "serde", feature = "utoipa-schema"))]
mod serialization_helper {

    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "utoipa-schema", derive(utoipa::ToSchema))]
    pub enum Event {
        Description {
            name: String,
            metric_type: super::MetricType,
            unit: Option<String>,
            description: String,
        },
        Metric {
            key: Key,
            op: super::MetricOperation,
        },
    }

    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "utoipa-schema", derive(utoipa::ToSchema))]
    pub struct Key {
        name: String,
        labels: Vec<(String, String)>,
    }

    impl From<&super::Event> for Event {
        fn from(event: &super::Event) -> Self {
            match event {
                super::Event::Description {
                    name,
                    metric_type,
                    unit,
                    description,
                } => Event::Description {
                    name: name.as_str().to_owned(),
                    metric_type: *metric_type,
                    unit: unit.map(|u| u.as_str().to_owned()),
                    description: description.to_string(),
                },
                super::Event::Metric { key, op } => Event::Metric {
                    key: Key {
                        name: key.name().to_string(),
                        labels: key
                            .labels()
                            .map(|label| (label.key().to_string(), label.value().to_string()))
                            .collect(),
                    },
                    op: *op,
                },
            }
        }
    }

    impl From<Event> for super::Event {
        fn from(event: Event) -> Self {
            use metrics::{
                Key,
                KeyName,
                Label,
                SharedString,
                Unit,
            };

            match event {
                Event::Description {
                    name,
                    metric_type,
                    unit,
                    description,
                } => super::Event::Description {
                    name: KeyName::from(name),
                    metric_type,
                    unit: unit.as_deref().and_then(Unit::from_string),
                    description: SharedString::from(description),
                },
                Event::Metric { key, op } => super::Event::Metric {
                    key: Key::from_parts(
                        key.name,
                        key.labels
                            .into_iter()
                            .map(|(k, v)| Label::new(SharedString::from(k), SharedString::from(v)))
                            .collect::<Vec<_>>(),
                    ),
                    op,
                },
            }
        }
    }

    #[cfg(feature = "serde")]
    impl serde::Serialize for super::Event {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let other = Event::from(self);
            other.serialize(serializer)
        }
    }

    #[cfg(feature = "serde")]
    impl<'de> serde::Deserialize<'de> for super::Event {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let other = Event::deserialize(deserializer)?;
            Ok(super::Event::from(other))
        }
    }

    #[cfg(feature = "utoipa-schema")]
    impl utoipa::__dev::ComposeSchema for super::Event {
        fn compose(
            generics: Vec<utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>>,
        ) -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
            <Event as utoipa::__dev::ComposeSchema>::compose(generics)
        }
    }

    #[cfg(feature = "utoipa-schema")]
    impl utoipa::ToSchema for super::Event {
        fn name() -> std::borrow::Cow<'static, str> {
            <Event as utoipa::ToSchema>::name()
        }
        fn schemas(schemas: &mut Vec<(String, utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>)>) {
            <Event as utoipa::ToSchema>::schemas(schemas);
        }
    }
}
