use crate::log;
use metrics_exporter_wasm::types::{self, asn1};

pub fn run() {
    serialization();
    deserialization();
}

const N: usize = 1000;

fn serialization() {
    use crate::benchmark::bench_env;

    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        let result = bench_env(data.clone(), move |events| {
            types::asn1::Events::from(events)
                .serialize_with_asn1rs()
                .expect("failed to serialize events")
        });
        log!("| asn1rs serialize {i} | {result}");
    }
}

fn deserialization() {
    use crate::benchmark::bench_env;

    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        {
            let data = asn1::Events::from(data.clone())
                .serialize_with_asn1rs()
                .unwrap();
            let result = bench_env(data, move |data| {
                types::asn1::Events::deserialize_with_asn1rs(&data)
                    .expect("failed to serialize data")
            });
            log!("| asn1rs serialize {i} | {result}");
        }
    }
}

fn events_1(n: usize) -> Vec<types::Event> {
    use types::*;
    let event = Event::Metric {
        key: metrics::Key::from_parts("hello", &[("hello", "world")]),
        op: MetricOperation::SetCounter(25),
    };
    (0..n).map(|_| event.clone()).collect()
}

fn events_2(n: usize) -> Vec<types::Event> {
    use types::*;
    let event = Event::Metadata {
        name: "hello".to_string().into(),
        metric_type: MetricType::Gauge,
        unit: Some(metrics::Unit::Bytes),
        description: "hello world".to_string().into(),
    };
    (0..n).map(|_| event.clone()).collect()
}
