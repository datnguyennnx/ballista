use std::net::SocketAddr;
use tracing::info;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use ballista::model::config;
use ballista::controller::router::create_router;
use ballista::middleware::{
    log_request, 
    create_cors_layer,
    init_logging,
};

#[tokio::main]
async fn main() {
    // Load configuration
    let app_config = config::load_config();
    
    // Initialize logging with improved format
    init_logging(&app_config.server.log_level);

    // Create the application router
    let (router, _state) = create_router();

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(["Content-Type", "Authorization"])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(86400));

    // Apply middleware in the correct order
    let app = router
        // Apply CORS directly to the router
        .layer(cors)
        // Apply request logging middleware
        .layer(axum::middleware::from_fn(log_request));

    // Get the server address from configuration
    let addr: SocketAddr = format!("{}:{}", app_config.server.host, app_config.server.port)
        .parse()
        .unwrap();

    // Start the server with a more informative message
    info!("🚀 Server starting on http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}