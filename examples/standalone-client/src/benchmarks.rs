use crate::util::benchmark::bench_env;
use metrics_exporter_wasm::{
    Event,
    Events,
    MetricOperation,
    MetricType,
};

pub fn run() {
    asn_serialization();
    asn_deserialization();
}

const N: u64 = 1000;

fn asn_serialization() {
    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        let result = bench_env(data.clone(), move |events| {
            Events::from(events)
                .serialize_with_asn1rs()
                .expect("failed to serialize events")
        });
        tracing::info!("| asn1rs serialize {i} | {result}");
    }
}

fn asn_deserialization() {
    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        {
            let data = Events::from(data.clone()).serialize_with_asn1rs().unwrap();
            let result = bench_env(data, move |data| {
                Events::deserialize_with_asn1rs(&data).expect("failed to serialize data")
            });
            tracing::info!("| asn1rs serialize {i} | {result}");
        }
    }
}

fn events_1(n: u64) -> Vec<Event> {
    (0..n)
        .map(|i| Event::Metric {
            key: metrics::Key::from_parts("hello", &[("hello", "world")]),
            op: MetricOperation::SetCounter(i),
        })
        .collect()
}

fn events_2(n: u64) -> Vec<Event> {
    let event = Event::Description {
        name: "hello".to_string().into(),
        metric_type: MetricType::Gauge,
        unit: Some(metrics::Unit::Bytes),
        description: "hello world".to_string().into(),
    };
    (0..n).map(|_| event.clone()).collect()
}
