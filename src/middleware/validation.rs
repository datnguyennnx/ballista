use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Middleware for validating JSON body size
pub async fn validate_json_body_size(
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Get content length from headers
    let content_length = request
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);

    // Define maximum size (2MB)
    let max_size = 2 * 1024 * 1024;

    // Check if content length exceeds the limit
    if content_length > max_size {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(json!({
                "status": "error",
                "message": format!("Request body too large, maximum size is {} bytes", max_size)
            })),
        ));
    }

    // Continue with the request if everything is fine
    Ok(next.run(request).await)
}

/// Middleware for validating required API keys
pub async fn validate_api_key(
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Skip validation for certain paths
    let path = request.uri().path();
    if path.starts_with("/api/health") || path.starts_with("/ws") {
        return Ok(next.run(request).await);
    }

    // Get API key from environment
    let expected_api_key = std::env::var("API_KEY").ok();

    // If API key is set, validate it
    if let Some(expected_key) = expected_api_key {
        if !expected_key.is_empty() {
            let api_key = request
                .headers()
                .get("x-api-key")
                .and_then(|value| value.to_str().ok());

            // If API key is missing or incorrect, return error
            if api_key != Some(&expected_key) {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "status": "error",
                        "message": "Invalid or missing API key"
                    })),
                ));
            }
        }
    }

    // Continue with the request if everything is fine
    Ok(next.run(request).await)
} 