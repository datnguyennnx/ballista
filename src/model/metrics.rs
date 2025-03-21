use std::time::Duration;
use std::collections::HashMap;

/// Represents metrics collected during testing
#[derive(Debug, Clone)]
pub struct Metrics {
    pub requests: u32,
    pub success: u32,
    pub errors: u32,
    pub durations: Vec<Duration>,
    pub status_codes: HashMap<u16, u32>,
    pub error_messages: Vec<String>,
}

/// Represents a summary of metrics
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub total_requests: u32,
    pub successful_requests: u32,
    pub failed_requests: u32,
    pub avg_time_ms: f64,
    pub min_time_ms: f64,
    pub max_time_ms: f64,
    pub status_codes: HashMap<u16, u32>,
}

/// Create new metrics
pub fn new_metrics() -> Metrics {
    Metrics {
        requests: 0,
        success: 0,
        errors: 0,
        durations: Vec::new(),
        status_codes: HashMap::new(),
        error_messages: Vec::new(),
    }
}

/// Add a request to metrics
pub fn add_request(metrics: &mut Metrics, duration: Duration, status: u16, error: Option<&str>) {
    metrics.requests += 1;
    if status >= 200 && status < 400 {
        metrics.success += 1;
    } else {
        metrics.errors += 1;
        if let Some(error_msg) = error {
            metrics.error_messages.push(error_msg.to_string());
        }
    }
    
    metrics.durations.push(duration);
    *metrics.status_codes.entry(status).or_insert(0) += 1;
}

/// Calculate summary from metrics
pub fn calculate_summary(metrics: &Metrics) -> MetricsSummary {
    let total_requests = metrics.requests;
    let successful_requests = metrics.success;
    let failed_requests = metrics.errors;
    
    let durations_ms: Vec<f64> = metrics.durations.iter()
        .map(|d| d.as_secs_f64() * 1000.0)
        .collect();
    
    let avg_time_ms = if !durations_ms.is_empty() {
        durations_ms.iter().sum::<f64>() / durations_ms.len() as f64
    } else {
        0.0
    };
    
    let min_time_ms = durations_ms.iter().copied().fold(f64::INFINITY, f64::min);
    let max_time_ms = durations_ms.iter().copied().fold(0.0, f64::max);
    
    let min_time_ms = if min_time_ms == f64::INFINITY { 0.0 } else { min_time_ms };
    
    let status_codes = metrics.status_codes.clone();
    
    MetricsSummary {
        total_requests,
        successful_requests,
        failed_requests,
        avg_time_ms,
        min_time_ms,
        max_time_ms,
        status_codes,
    }
} 