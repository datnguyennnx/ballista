use std::env;
use tower_http::cors::{CorsLayer, AllowOrigin};
use axum::http::Method;
use axum::http::header::{HeaderName, CONTENT_TYPE, AUTHORIZATION, ACCEPT};

/// Create a CORS middleware layer configured from environment variables
pub fn create_cors_layer() -> CorsLayer {
    // Get CORS allowed origins from environment
    let allowed_origins = env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    // Begin building the CORS layer
    let mut cors = CorsLayer::new();
    
    // Configure allowed origins
    if allowed_origins == "*" {
        cors = cors.allow_origin(AllowOrigin::any());
    } else {
        let origins = allowed_origins
            .split(',')
            .filter_map(|origin| origin.parse().ok())
            .collect::<Vec<_>>();
        
        cors = cors.allow_origin(AllowOrigin::list(origins));
    }
    
    // Configure standard methods
    cors = cors.allow_methods([
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
        Method::PATCH,
    ]);
    
    // Configure standard headers
    cors = cors.allow_headers([
        CONTENT_TYPE,
        AUTHORIZATION,
        ACCEPT,
        HeaderName::from_static("x-requested-with"),
    ]);
    
    // Set max age and allow credentials
    cors = cors
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(
            env::var("CORS_MAX_AGE")
                .unwrap_or_else(|_| "86400".to_string())
                .parse()
                .unwrap_or(86400),
        ));
        
    cors
}