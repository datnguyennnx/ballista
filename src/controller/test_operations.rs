use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use crate::model::state::AppState;
use crate::view::response::create_api_response;

/// Get all test results
pub async fn get_all_test_results(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let results = state.get_all_test_results().await;
    Json(create_api_response(
        true,
        "Test results retrieved".to_string(),
        Some(results),
    ))
} 