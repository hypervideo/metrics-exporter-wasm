use crate::util::benchmark::bench_env;
use metrics_exporter_wasm::{
    Event,
    Events,
    MetricOperation,
    MetricType,
};

pub fn run() {
    asn_serialization();
    asn_serialization_brotli();
    asn_serialization_zstd();
    asn_deserialization();
}

const N: u64 = 1000;

fn asn_serialization() {
    use metrics_exporter_wasm::WasmMetricsEncode as _;
    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        let result = bench_env(data.clone(), move |events| {
            Events::from(events).encode().expect("failed to serialize events")
        });
        let size = Events::from(data).encode().unwrap().len();
        tracing::info!("| asn1rs serialize {i} | {result} | {size}B");
    }
}

fn asn_serialization_brotli() {
    use metrics_exporter_wasm::WasmMetricsEncodeBrotli as _;
    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        let result = bench_env(data.clone(), move |events| {
            Events::from(events).encode().expect("failed to serialize events")
        });
        let size = Events::from(data).encode().unwrap().len();
        tracing::info!("| asn1rs serialize brotli {i} | {result} | {size}B");
    }
}

fn asn_serialization_zstd() {
    use metrics_exporter_wasm::WasmMetricsEncodeZstd as _;
    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        let result = bench_env(data.clone(), move |events| {
            Events::from(events).encode().expect("failed to serialize events")
        });
        let size = Events::from(data).encode().unwrap().len();
        tracing::info!("| asn1rs serialize zstd {i} | {result} | {size}B");
    }
}

fn asn_deserialization() {
    use metrics_exporter_wasm::{
        WasmMetricsDecode as _,
        WasmMetricsEncode as _,
    };

    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        {
            let data = Events::from(data.clone()).encode().unwrap();
            let result = bench_env(data, move |data| {
                Events::decode(&data).expect("failed to serialize data")
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
