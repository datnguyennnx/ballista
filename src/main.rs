use tokio::net::TcpListener;
use ballista::api::server::create_api_server;
use ballista::core::app::AppState;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Initialize shared application state
    let app_state = Arc::new(AppState::new());
    
    // Create and run API server
    let app = create_api_server(app_state).await;
    
    // Bind to port 3001
    match TcpListener::bind("0.0.0.0:3001").await {
        Ok(listener) => {
            println!("API server listening on http://localhost:3001");
            
            // Run the server
            if let Err(e) = axum::serve(listener, app).await {
                eprintln!("Server error: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Failed to bind to port 3001: {}", e);
            std::process::exit(1);
        }
    }
}
