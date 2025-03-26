use std::time::Duration;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

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

/// Thread-safe metrics collection structure
#[derive(Clone)]
pub struct OptimizedMetrics {
    pub requests_completed: Arc<AtomicU32>,
    pub errors: Arc<AtomicU32>,
    pub durations: Arc<Mutex<Vec<Duration>>>,
    pub status_codes: Arc<Mutex<HashMap<u16, u32>>>,
}

impl OptimizedMetrics {
    pub fn new() -> Self {
        Self {
            requests_completed: Arc::new(AtomicU32::new(0)),
            errors: Arc::new(AtomicU32::new(0)),
            durations: Arc::new(Mutex::new(Vec::new())),
            status_codes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Updates metrics with a new request result
    pub async fn update(&self, status: u16, duration: Duration, error: bool) {
        // Update atomic counters
        self.requests_completed.fetch_add(1, Ordering::SeqCst);
        if error {
            self.errors.fetch_add(1, Ordering::SeqCst);
        }

        // Update durations
        let mut durations = self.durations.lock().await;
        durations.push(duration);

        // Update status codes
        let mut status_codes = self.status_codes.lock().await;
        *status_codes.entry(status).or_insert(0) += 1;
    }

    /// Creates a snapshot of current metrics
    pub async fn snapshot(&self) -> TestMetricsSnapshot {
        let completed = self.requests_completed.load(Ordering::SeqCst);
        let errors = self.errors.load(Ordering::SeqCst);
        let durations = self.durations.lock().await.clone();
        let status_codes = self.status_codes.lock().await.clone();

        TestMetricsSnapshot {
            requests_completed: completed,
            errors,
            durations,
            status_codes,
        }
    }

    /// Resets all metrics
    pub async fn reset(&self) {
        self.requests_completed.store(0, Ordering::SeqCst);
        self.errors.store(0, Ordering::SeqCst);
        let mut durations = self.durations.lock().await;
        durations.clear();
        let mut status_codes = self.status_codes.lock().await;
        status_codes.clear();
    }
}

/// Snapshot of test metrics at a point in time
#[derive(Clone)]
pub struct TestMetricsSnapshot {
    pub requests_completed: u32,
    pub errors: u32,
    pub durations: Vec<Duration>,
    pub status_codes: HashMap<u16, u32>,
}

impl TestMetricsSnapshot {
    /// Calculates average response time
    pub fn average_response_time(&self) -> f64 {
        if self.durations.is_empty() {
            return 0.0;
        }
        let total: Duration = self.durations.iter().sum();
        total.as_secs_f64() / self.durations.len() as f64
    }

    /// Calculates error rate
    pub fn error_rate(&self) -> f64 {
        if self.requests_completed == 0 {
            return 0.0;
        }
        self.errors as f64 / self.requests_completed as f64
    }

    /// Calculates requests per second
    pub fn requests_per_second(&self) -> f64 {
        if self.durations.is_empty() {
            return 0.0;
        }
        let total_duration: Duration = self.durations.iter().sum();
        if total_duration.as_secs_f64() == 0.0 {
            return 0.0;
        }
        self.requests_completed as f64 / total_duration.as_secs_f64()
    }
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