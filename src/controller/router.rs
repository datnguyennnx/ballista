use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

use crate::model::state::AppState;
use crate::controller::health::health_check;
use crate::controller::{
    load_test_controller::start_load_test,
    stress_test_controller::start_stress_test,
    test_operations::get_all_test_results,
    websocket::handle_ws,
};

/// Create a new router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Health endpoint
        .route("/api/health", get(health_check))
        
        // Test endpoints
        .route("/api/tests", get(get_all_test_results))
        .route("/api/load-test", post(start_load_test))
        .route("/api/stress-test", post(start_stress_test))
        
        // WebSocket endpoint
        .route("/ws", get(handle_ws))
        
        // Add state to router
        .with_state(state)
} 