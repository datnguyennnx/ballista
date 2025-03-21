use axum::{
    response::IntoResponse,
    Json,
    http::StatusCode,
};

use crate::view::response::create_api_response;

/// Handler for the health check endpoint
pub async fn health_check() -> impl IntoResponse {
    // In a real application, we might check database connections,
    // external services, etc. here before returning healthy.
    let is_healthy = true;
    
    if is_healthy {
        (
            StatusCode::OK, 
            Json(create_api_response(true, "Service is healthy".to_string(), Some("ok".to_string())))
        )
    } else {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(create_api_response(false, "Service is unhealthy".to_string(), None::<String>))
        )
    }
} 