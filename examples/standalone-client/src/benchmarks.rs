#![allow(clippy::single_element_loop)]

use crate::util::benchmark::bench_env;
use metrics_exporter_wasm::{
    Event,
    Events,
    MetricOperation,
    MetricType,
    RecordedEvents,
};

pub fn run() {
    asn_serialization();
    asn_serialization_brotli();
    asn_serialization_zstd();
    asn_deserialization();
}

const N: u64 = 1000;

fn asn_serialization() {
    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        let result = bench_env(data.clone(), move |events| {
            Events::from(events).encode().expect("failed to serialize events")
        });
        let size = Events::from(data).encode().unwrap().len();
        tracing::info!("| asn1rs serialize {i} | {result} | {size}B");
    }
    for (i, data) in [(3, events_recording_events(N))] {
        let result = bench_env(data.clone(), move |events| {
            events.encode().expect("failed to serialize events")
        });
        let size = data.encode().unwrap().len();
        tracing::info!("| asn1rs serialize {i} | {result} | {size}B");
    }
}

fn asn_serialization_brotli() {
    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        let result = bench_env(data.clone(), move |events| {
            Events::from(events)
                .encode_and_compress_br()
                .expect("failed to serialize events")
        });
        let size = Events::from(data).encode_and_compress_br().unwrap().len();
        tracing::info!("| asn1rs serialize brotli {i} | {result} | {size}B");
    }
    for (i, data) in [(3, events_recording_events(N))] {
        let result = bench_env(data.clone(), move |events| {
            events.encode_and_compress_br().expect("failed to serialize events")
        });
        let size = data.encode_and_compress_br().unwrap().len();
        tracing::info!("| asn1rs serialize brotli {i} | {result} | {size}B");
    }
}

fn asn_serialization_zstd() {
    const COMPRESSION_LEVEL: u8 = 3;

    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        let result = bench_env(data.clone(), move |events| {
            Events::from(events)
                .encode_and_compress_zstd_external(3)
                .expect("failed to serialize events")
        });
        let size = Events::from(data.clone())
            .encode_and_compress_zstd_external(COMPRESSION_LEVEL)
            .unwrap()
            .len();
        tracing::info!("| asn1rs serialize zstd {i} | {result} | {size}B");

        let compressed = Events::from(data)
            .encode_and_compress_zstd_external(COMPRESSION_LEVEL)
            .unwrap();
        let result = bench_env(compressed, move |compressed| {
            use wasm_bindgen::prelude::*;
            use web_sys::js_sys::Uint8Array;
            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = zstd)]
                fn decompress(buf: Uint8Array) -> Uint8Array;
            }
            let decompressed = decompress(Uint8Array::from(compressed.as_slice()));
            Events::decode(&decompressed.to_vec()).unwrap()
        });
        tracing::info!("| asn1rs roundtrip zstd {i} | {result}");
    }

    for (i, data) in [(3, events_recording_events(N))] {
        let result = bench_env(data.clone(), move |events| {
            events
                .encode_and_compress_zstd_external(3)
                .expect("failed to serialize events")
        });
        let size = data
            .clone()
            .encode_and_compress_zstd_external(COMPRESSION_LEVEL)
            .unwrap()
            .len();
        tracing::info!("| asn1rs serialize zstd {i} | {result} | {size}B");

        let compressed = data.encode_and_compress_zstd_external(COMPRESSION_LEVEL).unwrap();
        let result = bench_env(compressed, move |compressed| {
            use wasm_bindgen::prelude::*;
            use web_sys::js_sys::Uint8Array;
            #[wasm_bindgen]
            extern "C" {
                #[wasm_bindgen(js_namespace = zstd)]
                fn decompress(buf: Uint8Array) -> Uint8Array;
            }
            let decompressed = decompress(Uint8Array::from(compressed.as_slice()));
            RecordedEvents::decode(&decompressed.to_vec()).unwrap()
        });
        tracing::info!("| asn1rs roundtrip zstd {i} | {result}");
    }
}

fn asn_deserialization() {
    for (i, data) in [(1, events_1(N)), (2, events_2(N))] {
        {
            let data = Events::from(data.clone()).encode().unwrap();
            let result = bench_env(data, move |data| {
                Events::decode(&data).expect("failed to serialize data")
            });
            tracing::info!("| asn1rs serialize {i} | {result}");
        }
    }
    for (i, data) in [(3, events_recording_events(N))] {
        {
            let data = data.clone().encode().unwrap();
            let result = bench_env(data, move |data| {
                RecordedEvents::decode(&data).expect("failed to serialize data")
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

fn events_recording_events(n: u64) -> RecordedEvents {
    RecordedEvents::from(events_1(n))
}
