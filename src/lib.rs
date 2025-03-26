pub mod controller;
pub mod http;
pub mod middleware;
pub mod model;
pub mod view;

// Re-export key types for easier access
pub use model::{
    config::AppConfig,
    error::AppError,
    state::AppState,
    test::{
        TestConfig, TestResult, TestStatus, TestType, TestMetrics, TestUpdate,
        ApiTestConfig, LoadTestConfig, StressTestConfig, ApiTest, // Use ApiTest
        RequestResult, ApiRequestResult
    },
    time_series::TimeSeriesPoint,
};

pub use controller::router::create_router;