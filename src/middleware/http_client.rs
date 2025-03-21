use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Log outgoing HTTP requests made by the client
pub fn log_outgoing_request(method: &str, url: &str) -> Uuid {
    let request_id = Uuid::new_v4();
    
    info!(
        request_id = %request_id,
        method = %method,
        url = %url,
        "Outgoing request"
    );
    
    // Return the request ID for later correlation with response
    request_id
}

/// Log responses from outgoing HTTP requests
pub fn log_outgoing_response(request_id: Uuid, url: &str, status: u16, elapsed_ms: u128) {
    match status {
        200..=299 => {
            info!(
                request_id = %request_id,
                url = %url,
                status = %status,
                elapsed_ms = %elapsed_ms,
                "Outgoing request completed successfully"
            );
        }
        300..=399 => {
            debug!(
                request_id = %request_id,
                url = %url,
                status = %status,
                elapsed_ms = %elapsed_ms,
                "Outgoing request redirected"
            );
        }
        400..=499 => {
            warn!(
                request_id = %request_id,
                url = %url,
                status = %status,
                elapsed_ms = %elapsed_ms,
                "Outgoing request client error"
            );
        }
        _ => {
            error!(
                request_id = %request_id,
                url = %url,
                status = %status,
                elapsed_ms = %elapsed_ms,
                "Outgoing request server error"
            );
        }
    }
} 