use axum::Json;
use serde_json::{json, Value};

// Pure function for health check
pub async fn health_check() -> Json<Value> {
    Json(json!({
        "success": true,
        "message": "API is running"
    }))
} 