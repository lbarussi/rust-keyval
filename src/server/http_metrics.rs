use axum::{routing::get, Router};
use std::net::SocketAddr;
use tokio::net::TcpListener;

use crate::server::metrics_prom;

async fn metrics_handler() -> String {
    String::from_utf8_lossy(&metrics_prom::gather()).to_string()
}

pub async fn serve(bind: &str) {
    let app = Router::new().route("/metrics", get(metrics_handler));

    let addr: SocketAddr = bind.parse().expect("invalid METRICS_BIND");
    let listener = TcpListener::bind(addr)
        .await
        .expect("failed to bind METRICS_BIND");

    axum::serve(listener, app)
        .await
        .expect("metrics server failed");
}