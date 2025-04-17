use super::generated;
use crate::{
    RecordedEvent,
    RecordedEvents,
};
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

impl From<RecordedEvents> for generated::RecordedEvents {
    fn from(value: RecordedEvents) -> Self {
        let RecordedEvents {
            recording_started_at: base_time,
            events,
        } = value;

        let unix_epoch = Utc.timestamp_opt(0, 0).unwrap();
        let duration = base_time.signed_duration_since(unix_epoch).to_std().unwrap_or_default();
        let recording_started_at = generated::Timestamp {
            seconds: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        };

        Self {
            recording_started_at,
            events: events
                .into_iter()
                .map(|event| event.into_asn_with_base_time(base_time))
                .collect(),
        }
    }
}

impl From<generated::RecordedEvents> for RecordedEvents {
    fn from(value: generated::RecordedEvents) -> Self {
        let generated::RecordedEvents {
            recording_started_at,
            events,
        } = value;

        let duration = chrono::Duration::seconds(recording_started_at.seconds as i64)
            + chrono::Duration::nanoseconds(recording_started_at.nanos as i64);
        let unix_epoch = Utc.timestamp_opt(0, 0).unwrap();
        let recording_started_at = unix_epoch + duration;

        Self {
            recording_started_at,
            events: events
                .into_iter()
                .map(|event| RecordedEvent::from_asn_with_base_time(event, recording_started_at))
                .collect(),
        }
    }
}
