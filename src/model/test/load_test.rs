use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestConfig {
    pub target_url: String,
    pub concurrent_users: Option<u32>,
    pub num_requests: u32,
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

pub fn create_test_metrics(
    requests_completed: u32,
    total_requests: u32,
    durations: &[Duration],
    status_codes: HashMap<u16, u32>,
    errors: u32,
) -> TestMetrics {
    let total_duration = durations.iter()
        .map(|d| d.as_secs_f64())
        .sum::<f64>();
    
    let avg_response_time = if !durations.is_empty() {
        total_duration / durations.len() as f64
    } else {
        0.0
    };
    
    let min_response_time = durations.iter()
        .map(|d| d.as_secs_f64())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);
    
    let max_response_time = durations.iter()
        .map(|d| d.as_secs_f64())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);
    
    let error_rate = if requests_completed > 0 {
        (errors as f64 / requests_completed as f64) * 100.0
    } else {
        0.0
    };
    
    let requests_per_second = if total_duration > 0.0 {
        requests_completed as f64 / total_duration
    } else {
        0.0
    };
    
    TestMetrics {
        requests_completed,
        total_requests,
        average_response_time: avg_response_time,
        min_response_time,
        max_response_time,
        error_rate,
        requests_per_second,
        status_codes,
    }
} 