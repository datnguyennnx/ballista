pub mod error;
pub mod test;
pub mod metrics;
pub mod state;
pub mod utils;
pub mod config;
pub mod time_series;
// Re-export common types from model
pub use error::AppError;
pub use test::{TestConfig, TestResult, TestStatus, TestType, TestMetrics, TestUpdate, ApiTest};
pub use metrics::{Metrics, new_metrics, add_request, calculate_summary};
pub use state::AppState;
pub use utils::formatters::format_duration; 
pub use config::{load_config, load_server_config, load_cors_config, load_websocket_config, load_test_runner_config};
pub use time_series::{TimeSeriesPoint, TimeSeriesTracker};