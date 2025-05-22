use super::{
    Asn1Decode,
    Asn1Encode,
    Error,
    Events,
    Result,
};
use crate::Event;
use asn1rs::prelude::*;

impl Asn1Encode for Events {
    /// Serialize the events using asn1.
    fn encode(&self) -> Result<Vec<u8>> {
        let mut writer = UperWriter::default();
        writer
            .write(self)
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;
        Ok(writer.into_bytes_vec())
    }
}

impl Asn1Decode for Events {
    /// Deserialize from asn1.
    fn decode(data: &[u8]) -> Result<Self> {
        let mut reader = UperReader::from(Bits::from(data));
        reader
            .read::<Events>()
            .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

impl From<Events> for Vec<Event> {
    fn from(value: Events) -> Self {
        value.0.into_iter().map(Event::from).collect()
    }
}

impl From<Vec<Event>> for Events {
    fn from(value: Vec<Event>) -> Self {
        Events(value.into_iter().map(Into::into).collect())
    }
}
