use super::generated;
use crate::MetricOperation;

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
