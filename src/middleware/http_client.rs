use tracing::info;
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
