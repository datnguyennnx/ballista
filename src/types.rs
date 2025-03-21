use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use serde_json::Value;

// Test Configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestConfig {
    pub target_url: String,
    pub num_requests: u32,
    pub concurrency: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestConfig {
    pub target_url: String,
    pub duration_secs: u64,
    pub concurrency: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTestConfig {
    pub target_url: String,
    pub test_suite_path: String,
}

// Test Results and Updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub id: String,
    pub test_type: TestType,
    pub status: TestStatus,
    pub metrics: Option<TestMetrics>,
    pub error: Option<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetrics {
    pub requests_completed: u32,
    pub total_requests: u32,
    pub avg_response_time: f64,
    pub min_response_time: Option<f64>,
    pub max_response_time: Option<f64>,
    pub median_response_time: Option<f64>,
    pub p95_response_time: Option<f64>,
    pub status_codes: HashMap<u16, u32>,
    pub errors: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TestType {
    Load,
    Stress,
    Api,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TestStatus {
    Started,
    Running,
    Completed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUpdate {
    pub id: String,
    pub test_type: TestType,
    pub status: TestStatus,
    pub progress: f32,
    pub metrics: Option<TestMetrics>,
    pub error: Option<String>,
    pub timestamp: i64,
}

// API Test Specific Types
#[derive(Debug, Clone, Deserialize)]
pub struct ApiTest {
    pub name: String,
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Value>,
    pub expected_status: u16,
    pub expected_body: Option<Value>,
}

// Error Types
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Invalid test configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Test execution failed: {0}")]
    ExecutionError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

// Helper functions for creating test results and updates
pub fn create_test_result(
    id: String,
    test_type: TestType,
    status: TestStatus,
    metrics: Option<TestMetrics>,
    error: Option<String>,
) -> TestResult {
    TestResult {
        id,
        test_type,
        status,
        metrics,
        error,
        timestamp: chrono::Utc::now().timestamp(),
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
        timestamp: chrono::Utc::now().timestamp(),
    }
}

pub fn create_test_metrics(
    requests_completed: u32,
    total_requests: u32,
    durations: &[Duration],
    status_codes: HashMap<u16, u32>,
    errors: u32,
) -> TestMetrics {
    let durations_ms: Vec<f64> = durations.iter()
        .map(|d| d.as_secs_f64() * 1000.0)
        .collect();

    let avg = if !durations_ms.is_empty() {
        durations_ms.iter().sum::<f64>() / durations_ms.len() as f64
    } else {
        0.0
    };

    let mut sorted_durations = durations_ms.clone();
    sorted_durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

    TestMetrics {
        requests_completed,
        total_requests,
        avg_response_time: avg,
        min_response_time: sorted_durations.first().copied(),
        max_response_time: sorted_durations.last().copied(),
        median_response_time: get_percentile(&sorted_durations, 50.0),
        p95_response_time: get_percentile(&sorted_durations, 95.0),
        status_codes,
        errors,
    }
}

fn get_percentile(sorted_values: &[f64], percentile: f64) -> Option<f64> {
    if sorted_values.is_empty() {
        return None;
    }
    let index = ((sorted_values.len() as f64 * percentile / 100.0).round() as usize)
        .saturating_sub(1)
        .min(sorted_values.len() - 1);
    Some(sorted_values[index])
} 