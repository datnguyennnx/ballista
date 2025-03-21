use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

use crate::model::state::AppState;
use crate::controller::health::health_check;
use crate::controller::test::{get_all_test_results, start_load_test, start_stress_test, start_api_test};
use crate::controller::websocket::ws_handler;

/// Create a new router with all routes
pub fn create_router() -> (Router, Arc<AppState>) {
    // Create application state
    let (state, _) = AppState::new();
    let state = Arc::new(state);

    // Create the router with all routes
    let router = Router::new()
        // Health endpoint
        .route("/api/health", get(health_check))
        
        // Test endpoints
        .route("/api/tests", get(get_all_test_results))
        .route("/api/load-test", post(start_load_test))
        .route("/api/stress-test", post(start_stress_test))
        .route("/api/api-test", post(start_api_test))
        
        // WebSocket endpoint
        .route("/ws", get(ws_handler))
        
        // Add state to router
        .with_state(Arc::clone(&state));
    
    (router, state)
} 