pub mod response;
pub mod formatter;

// Re-export common view components
pub use response::{ApiResponse, create_api_response};
pub use formatter::{format_test_results, format_metrics}; 