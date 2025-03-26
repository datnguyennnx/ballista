use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use serde_json::Value;
use std::fmt;
use chrono;

pub mod api_test;
pub mod load_test;
pub mod stress_test;

// Re-export types with unique names to avoid conflicts
pub use api_test::ApiTestConfig;
pub use load_test::LoadTestConfig;
pub use stress_test::StressTestConfig;
pub use crate::model::time_series::TimeSeriesPoint;
// Correctly re-export ApiTest from its submodule
pub use api_test::ApiTest; // Renamed from ApiTestRequest

// Common types used across all test types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub target_url: String,
    pub concurrent_users: u32,
    pub duration_secs: u32,
    pub num_requests: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestType {
    Load,
    Stress,
    Api,
}

impl fmt::Display for TestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestType::Load => write!(f, "Load"),
            TestType::Stress => write!(f, "Stress"),
            TestType::Api => write!(f, "Api"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TestStatus {
    Pending,
    Started,
    Running,
    Completed,
    Error,
}

impl fmt::Display for TestStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestStatus::Pending => write!(f, "Pending"),
            TestStatus::Started => write!(f, "Started"),
            TestStatus::Running => write!(f, "Running"),
            TestStatus::Completed => write!(f, "Completed"),
            TestStatus::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetrics {
    pub requests_completed: u32,
    pub total_requests: u32,
    pub average_response_time: f64,
    pub min_response_time: f64,
    pub max_response_time: f64,
    pub error_rate: f64,
    pub requests_per_second: f64,
    pub status_codes: HashMap<u16, u32>,
}

impl Default for TestMetrics {
    fn default() -> Self {
        TestMetrics {
            requests_completed: 0,
            total_requests: 0,
            average_response_time: 0.0,
            min_response_time: 0.0,
            max_response_time: 0.0,
            error_rate: 0.0,
            requests_per_second: 0.0,
            status_codes: HashMap::new(),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub id: String,
    pub test_type: TestType,
    pub status: TestStatus,
    pub progress: f32,
    pub metrics: Option<TestMetrics>,
    pub error: Option<String>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUpdate {
    pub id: String,
    pub test_type: TestType,
    pub status: TestStatus,
    pub progress: f32,
    pub metrics: Option<TestMetrics>,
    pub error: Option<String>,
}

// --- Result Structs ---

// Result for Load/Stress tests
#[derive(Debug, Clone)]
pub struct RequestResult {
    pub duration: Duration,
    pub status: u16,
}

// Result for API tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequestResult {
    pub duration: Duration,
    pub status: u16,
    pub json: Option<Value>,
}


// --- Pure functions for creating test entities ---
// (Kept for now, review later if needed)

pub fn create_test_result(
    id: String,
    test_type: TestType,
    status: TestStatus,
    progress: f32,
    metrics: Option<TestMetrics>,
    error: Option<String>,
) -> TestResult {
    TestResult {
        id,
        test_type,
        status,
        progress,
        metrics,
        error,
        start_time: chrono::Utc::now(),
        end_time: None,
    }
}

pub fn create_test_update(
    id: String,
    test_type: TestType,
    status: TestStatus,
    progress: f32,
    metrics: Option<TestMetrics>,
    error: Option<String>,
) -> TestUpdate {
    TestUpdate {
        id,
        test_type,
        status,
        progress,
        metrics,
        error,
    }
}

pub fn create_test_metrics(
    requests_completed: u32,
    total_requests: u32,
    durations: &[Duration],
    status_codes: HashMap<u16, u32>,
    errors: u32,
) -> TestMetrics {
    let total_duration_secs = durations.iter()
        .map(|d| d.as_secs_f64())
        .sum::<f64>();

    let avg_response_time_ms = if !durations.is_empty() {
        (total_duration_secs * 1000.0) / durations.len() as f64
    } else {
        0.0
    };

    let min_response_time_ms = durations.iter()
        .map(|d| d.as_secs_f64() * 1000.0)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);

    let max_response_time_ms = durations.iter()
        .map(|d| d.as_secs_f64() * 1000.0)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);

    let error_rate = if requests_completed > 0 {
        (errors as f64 / requests_completed as f64) * 100.0
    } else {
        0.0
    };

    let requests_per_second = if total_duration_secs > 0.0 {
        requests_completed as f64 / total_duration_secs
    } else {
        0.0
    };

    TestMetrics {
        requests_completed,
        total_requests,
        average_response_time: avg_response_time_ms,
        min_response_time: min_response_time_ms,
        max_response_time: max_response_time_ms,
        error_rate,
        requests_per_second,
        status_codes,
    }
}

// Helper functions for test configuration conversion
pub fn create_test_config_from_load(config: &LoadTestConfig) -> TestConfig {
    TestConfig {
        target_url: config.target_url.clone(),
        concurrent_users: config.concurrent_users.unwrap_or(1),
        duration_secs: 0,
        num_requests: config.num_requests,
    }
}

pub fn create_test_config_from_stress(config: &StressTestConfig) -> TestConfig {
    TestConfig {
        target_url: config.target_url.clone(),
        concurrent_users: config.concurrent_users,
        duration_secs: config.duration_secs,
        num_requests: 0,
    }
}