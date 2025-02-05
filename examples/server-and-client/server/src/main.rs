#[macro_use]
extern crate tracing;

use axum::{
    routing::get,
    Router,
};
use axum_prometheus::PrometheusMetricLayer;
use std::{
    net::SocketAddr,
    time::Duration,
};
use tower_http::services::ServeDir;

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

#[tokio::main]
async fn main() {
    init_logging();

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    let app = Router::<()>::new()
        .route("/fast", get(|| async {}))
        .route(
            "/slow",
            get(|| async {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }),
        )
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .fallback_service(ServeDir::new(".").append_index_html_on_directories(true))
        .layer(prometheus_layer);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
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
