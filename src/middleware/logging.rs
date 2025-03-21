use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use tracing_subscriber::prelude::*;
use chrono::{Local, Timelike, Datelike};

/// Format the current date and time in MM/DD/YYYY HH:MM format
fn format_date() -> String {
    let now = Local::now();
    format!(
        "{:02}/{:02}/{} {:02}:{:02}",
        now.month(), now.day(), now.year(),
        now.hour(), now.minute()
    )
}

/// Middleware to log all HTTP requests with timing information
pub async fn log_request(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Generate a unique request ID
    let request_id = Uuid::new_v4().to_string().split('-').next().unwrap_or("").to_string();
    
    // Extract request information
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().unwrap_or("");
    
    // Format the date
    let date = format_date();
    
    // Log the incoming request - simplified format
    info!(
        "[{}] [{}] → {} {}{}", 
        date,
        request_id,
        method,
        path,
        if query.is_empty() { String::new() } else { format!("?{}", query) }
    );
    
    // Record start time
    let start = Instant::now();
    
    // Process the request
    let response = next.run(request).await;
    
    // Calculate elapsed time
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    
    // Format elapsed time in a human-readable way
    let elapsed_str = if elapsed_ms < 1 {
        format!("{}μs", elapsed.as_micros())
    } else if elapsed_ms < 1000 {
        format!("{}ms", elapsed_ms)
    } else {
        format!("{:.2}s", elapsed_ms as f64 / 1000.0)
    };
    
    // Extract status code
    let status = response.status();
    let status_code = status.as_u16();
    
    // Log the response with appropriate level based on status code
    match status_code {
        200..=299 => {
            info!(
                "[{}] [{}] ← {} ✓ {} {} {}", 
                date, request_id, status_code, elapsed_str, method, path
            );
        }
        300..=399 => {
            debug!(
                "[{}] [{}] ← {} ↪ {} {} {}", 
                date, request_id, status_code, elapsed_str, method, path
            );
        }
        400..=499 => {
            warn!(
                "[{}] [{}] ← {} ⚠ {} {} {}", 
                date, request_id, status_code, elapsed_str, method, path
            );
        }
        _ => {
            error!(
                "[{}] [{}] ← {} ✗ {} {} {}", 
                date, request_id, status_code, elapsed_str, method, path
            );
        }
    }
    
    Ok(response)
}

/// Initialize the tracing subscriber with a more readable format
pub fn init_logging(log_level: &str) {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)  // Don't include the target
        .without_time()      // We're adding our own time format
        .compact();          // Use compact format
        
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| log_level.into()))
        .with(fmt_layer)
        .init();
} 