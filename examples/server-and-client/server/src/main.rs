#[macro_use]
extern crate tracing;

use axum::{
    response::IntoResponse,
    routing::{
        get,
        post,
    },
    Router,
};
use metrics_exporter_wasm_core::{
    Event,
    Events,
};
use std::{
    net::SocketAddr,
    time::Duration,
};
use tower_http::{
    decompression::RequestDecompressionLayer,
    services::ServeDir,
};

fn init_logging() {
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_filter(tracing_subscriber::EnvFilter::from_default_env()),
        )
        .init();
}

async fn metrics() -> impl IntoResponse {
    match prometheus::TextEncoder::new().encode_to_string(&prometheus::default_registry().gather()) {
        Ok(s) => (hyper::StatusCode::OK, s),
        Err(e) => {
            error!("failed to encode metrics: {:?}", e);
            (
                hyper::StatusCode::INTERNAL_SERVER_ERROR,
                "failed to encode metrics".to_string(),
            )
        }
    }
}

async fn receive_metrics(data: axum::body::Bytes) -> hyper::StatusCode {
    match Events::deserialize_with_asn1rs(&data) {
        Ok(events) => {
            let events: Vec<Event> = events.into();
            for event in events {
                dbg!(&event);
                match event {
                    Event::Metadata {
                        name,
                        metric_type,
                        unit,
                        description,
                    } => match metric_type {
                        metrics_exporter_wasm_core::MetricType::Counter => {
                            metrics::with_recorder(|recorder| recorder.describe_counter(name, unit, description));
                        }
                        metrics_exporter_wasm_core::MetricType::Gauge => {
                            metrics::with_recorder(|recorder| recorder.describe_gauge(name, unit, description));
                        }
                        metrics_exporter_wasm_core::MetricType::Histogram => {
                            metrics::with_recorder(|recorder| recorder.describe_histogram(name, unit, description));
                        }
                    },
                    Event::Metric { key, op } => {
                        let metadata = {
                            static METADATA: metrics::Metadata<'static> =
                                metrics::Metadata::new("", metrics::Level::INFO, None);
                            &METADATA
                        };

                        match op {
                            metrics_exporter_wasm_core::MetricOperation::IncrementCounter(value) => {
                                metrics::with_recorder(|recorder| recorder.register_counter(&key, metadata))
                                    .increment(value);
                            }
                            metrics_exporter_wasm_core::MetricOperation::SetCounter(value) => {
                                metrics::with_recorder(|recorder| recorder.register_counter(&key, metadata))
                                    .absolute(value);
                            }
                            metrics_exporter_wasm_core::MetricOperation::IncrementGauge(value) => {
                                metrics::with_recorder(|recorder| recorder.register_gauge(&key, metadata))
                                    .increment(value);
                            }
                            metrics_exporter_wasm_core::MetricOperation::DecrementGauge(value) => {
                                metrics::with_recorder(|recorder| recorder.register_gauge(&key, metadata))
                                    .decrement(value);
                            }
                            metrics_exporter_wasm_core::MetricOperation::SetGauge(value) => {
                                metrics::with_recorder(|recorder| recorder.register_gauge(&key, metadata)).set(value);
                            }
                            metrics_exporter_wasm_core::MetricOperation::RecordHistogram(value) => {
                                metrics::with_recorder(|recorder| recorder.register_histogram(&key, metadata))
                                    .record(value);
                            }
                        }
                    }
                }
            }
            hyper::StatusCode::OK
        }
        Err(e) => {
            error!("failed to decode metrics: {:?}", e);
            hyper::StatusCode::BAD_REQUEST
        }
    }
}

fn metrics_test() {
    // By default `prometheus::default_registry()` is used.
    let recorder = metrics_prometheus::install();

    // Either use `metrics` crate interfaces.
    metrics::counter!("count", "whose" => "mine", "kind" => "owned").increment(1);
    metrics::counter!("count", "whose" => "mine", "kind" => "ref").increment(1);
    metrics::counter!("count", "kind" => "owned", "whose" => "dummy").increment(1);
    {
        let gauge = prometheus::Gauge::new("value", "help").unwrap();
        recorder.register_metric(gauge.clone());
        gauge.inc();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                gauge.inc();
            }
        });
    }
}

#[tokio::main]
async fn main() {
    init_logging();

    metrics_test();

    // -=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-

    let decompression_layer = RequestDecompressionLayer::new().br(true);

    let app = Router::<()>::new()
        .route("/fast", get(|| async {}))
        .route(
            "/slow",
            get(|| async {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }),
        )
        .route("/metrics", get(metrics))
        .route("/receive-metrics", post(receive_metrics))
        .layer(decompression_layer)
        .fallback_service(ServeDir::new(".").append_index_html_on_directories(true));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("starting on {:?}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind to address");

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install CTRL+C signal handler")
        })
        .await
        .expect("server failed to start");
}
