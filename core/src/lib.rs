//! asn.1 metrics implementation using the asn1rs crate.

mod asn;
mod event;
pub mod util_time;
#[cfg(feature = "compress-zstd-external")]
pub mod zstd_external;

pub use asn::{
    Error,
    Events,
    Result,
};
pub use event::{
    Event,
    MetricOperation,
    MetricType,
    RecordedEvent,
    RecordedEvents,
};
