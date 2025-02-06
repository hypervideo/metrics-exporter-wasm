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
use std::{
    net::SocketAddr,
    time::Duration,
};
use tower_http::{
    decompression::DecompressionLayer,
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

async fn receive_metrics() -> impl IntoResponse {
    hyper::StatusCode::OK
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

    let decomression_layer: DecompressionLayer = DecompressionLayer::new().br(true).deflate(true).gzip(true).zstd(true);

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
        .fallback_service(ServeDir::new(".").append_index_html_on_directories(true))
        .layer(decomression_layer);

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
