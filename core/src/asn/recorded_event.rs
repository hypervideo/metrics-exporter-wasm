use super::{
    generated,
    Asn1Decode,
    Asn1Encode,
    Error,
    Result,
};
use crate::{
    util_time,
    Event,
    RecordedEvent,
};
use asn1rs::prelude::*;
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

impl From<Event> for RecordedEvent {
    fn from(event: Event) -> Self {
        RecordedEvent {
            timestamp: util_time::now(),
            event,
        }
    }
}

impl From<RecordedEvent> for Event {
    fn from(event: RecordedEvent) -> Self {
        event.event
    }
}

// -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

impl generated::RecordedEvents {
    pub fn new(batch_start_time: DateTime<Utc>, events: Vec<RecordedEvent>) -> Self {
        let unix_epoch = Utc.timestamp_opt(0, 0).unwrap();
        let duration = batch_start_time
            .signed_duration_since(unix_epoch)
            .to_std()
            .unwrap_or_default();
        let recording_started_at = generated::Timestamp {
            seconds: duration.as_secs(),
            nanos: duration.subsec_nanos(),
        };

        Self {
            recording_started_at,
            events: events
                .into_iter()
                .map(|event| event.into_asn_with_base_time(batch_start_time))
                .collect(),
        }
    }
}

impl Asn1Encode for generated::RecordedEvents {
    /// Serialize the events using asn1.
    fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = UperWriter::default();
        writer
            .write(self)
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(writer.into_bytes_vec())
    }
}

impl Asn1Decode for generated::RecordedEvents {
    /// Deserialize from asn1.
    fn decode(data: &[u8]) -> Result<Self> {
        let mut reader = UperReader::from(Bits::from(data));
        reader
            .read::<generated::RecordedEvents>()
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

impl From<generated::RecordedEvents> for Vec<RecordedEvent> {
    fn from(value: generated::RecordedEvents) -> Self {
        let generated::RecordedEvents {
            recording_started_at,
            events,
        } = value;

        let duration = chrono::Duration::seconds(recording_started_at.seconds as i64)
            + chrono::Duration::nanoseconds(recording_started_at.nanos as i64);
        let unix_epoch = Utc.timestamp_opt(0, 0).unwrap();
        let recording_started_at = unix_epoch + duration;

        events
            .into_iter()
            .map(|event| RecordedEvent::from_asn_with_base_time(event, recording_started_at))
            .collect()
    }
}

impl From<Vec<Event>> for generated::RecordedEvents {
    fn from(events: Vec<Event>) -> Self {
        let now = util_time::now();
        let events = events
            .into_iter()
            .map(|event| RecordedEvent { event, timestamp: now })
            .collect();
        Self::new(now, events)
    }
}
