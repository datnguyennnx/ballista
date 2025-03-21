pub mod error;
pub mod test;
pub mod metrics;
pub mod state;
pub mod utils;

// Re-export common types from model
pub use error::AppError;
pub use test::{TestConfig, TestResult, TestStatus, TestType, TestMetrics, TestUpdate, ApiTest};
pub use metrics::{Metrics, new_metrics, add_request, calculate_summary};
pub use state::AppState;
pub use utils::formatters::format_duration; 