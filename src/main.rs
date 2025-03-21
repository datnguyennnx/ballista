use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Create router with all routes
    let (app, _state) = ballista::controller::create_router();
    
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
