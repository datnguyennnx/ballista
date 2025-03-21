use axum::{
    Router,
    routing::{get, post},
};
use tokio::sync::broadcast;
use std::sync::Arc;

pub mod websocket;
pub mod test;
pub mod health;

use websocket::ws_handler;
use test::{start_stress_test, start_load_test, start_api_test};
use health::health_check;

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<String>,
}

// Pure function to create router configuration
pub fn create_router_config() -> (Router, broadcast::Sender<String>) {
    // Create a channel for WebSocket broadcasts
    let (tx, _) = broadcast::channel(100);
    let state = Arc::new(AppState { tx: tx.clone() });

    let router = Router::new()
        // WebSocket route
        .route("/ws", get(ws_handler))
        // Health check route
        .route("/api/health", get(health_check))
        // Test routes
        .route("/api/load-test", post(start_load_test))
        .route("/api/stress-test", post(start_stress_test))
        .route("/api/api-test", post(start_api_test))
        .with_state(state);

    (router, tx)
} 