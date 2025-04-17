use super::generated;
use crate::Event;
use metrics::Key;

impl From<generated::Event> for Event {
    fn from(value: generated::Event) -> Self {
        use generated::{
            EventDescription,
            EventMetric,
            EventMetricKey,
        };
        match value {
            generated::Event::Description(EventDescription {
                key_name: name,
                metric_type,
                unit,
                description,
            }) => Event::Description {
                name: name.into(),
                metric_type: metric_type.into(),
                unit: unit.map(Into::into),
                description: description.into(),
            },

            generated::Event::Metric(EventMetric {
                key: EventMetricKey { name, label },
                op,
            }) => {
                let labels = label
                    .into_iter()
                    .map(|entry| metrics::Label::new(entry.key, entry.value))
                    .collect::<Vec<_>>();
                let key = Key::from_parts(name, labels);
                Event::Metric { key, op: op.into() }
            }
        }
    }
}

impl From<Event> for generated::Event {
    fn from(value: Event) -> Self {
        use generated::{
            EventDescription,
            EventMetric,
            EventMetricKey,
            EventMetricKeyLabel,
        };
        match value {
            Event::Description {
                name,
                metric_type,
                unit,
                description,
            } => generated::Event::Description(EventDescription {
                key_name: name.as_str().into(),
                metric_type: metric_type.into(),
                unit: unit.map(Into::into),
                description: description.to_string(),
            }),

            Event::Metric { key, op } => {
                let (key_name, key_labels) = key.into_parts();

                generated::Event::Metric(EventMetric {
                    key: EventMetricKey {
                        name: key_name.as_str().into(),
                        label: key_labels
                            .into_iter()
                            .map(|label| {
                                let (key, value) = label.into_parts();
                                EventMetricKeyLabel {
                                    key: key.to_string(),
                                    value: value.to_string(),
                                }
                            })
                            .collect(),
                    },
                    op: op.into(),
                })
            }
        }
    }
}
