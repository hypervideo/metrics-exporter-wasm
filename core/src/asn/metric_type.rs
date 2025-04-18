use super::generated;
use crate::MetricType;

impl From<generated::MetricType> for MetricType {
    fn from(value: generated::MetricType) -> Self {
        match value {
            generated::MetricType::Counter => MetricType::Counter,
            generated::MetricType::Gauge => MetricType::Gauge,
            generated::MetricType::Histogram => MetricType::Histogram,
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
