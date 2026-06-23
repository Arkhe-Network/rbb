use axum::{
    routing::{get, post},
    Router,
    response::IntoResponse,
    Json,
};
use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/compile", post(compile_handler))
        .route("/api/agent/run", post(agent_run_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "OK"
}

async fn compile_handler() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "mock compile" }))
}

async fn agent_run_handler() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "mock agent run" }))
}
