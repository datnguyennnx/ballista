use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::model::test::TestMetrics;

/// Time series data point that matches the frontend's TimeSeriesPoint interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: i64,
    pub requests_per_second: f64,
    pub average_response_time: f64,
    pub error_rate: f64,
}

impl TimeSeriesPoint {
    /// Create a new time series point from test metrics
    pub fn from_metrics(metrics: &TestMetrics, prev_metrics: Option<&TestMetrics>, elapsed_seconds: f64) -> Self {
        // Calculate requests per second based on current metrics and previous metrics
        let rps = if let Some(prev) = prev_metrics {
            let requests_diff = metrics.requests_completed as f64 - prev.requests_completed as f64;
            if elapsed_seconds > 0.0 {
                requests_diff / elapsed_seconds
            } else {
                0.0
            }
        } else {
            // If no previous metrics, calculate average based on total test time
            if elapsed_seconds > 0.0 {
                metrics.requests_completed as f64 / elapsed_seconds
            } else {
                0.0
            }
        };

        Self {
            timestamp: Utc::now().timestamp_millis(),
            requests_per_second: rps,
            average_response_time: metrics.average_response_time,
            error_rate: metrics.error_rate,
        }
    }
}

/// Helper struct to track and generate time series data
pub struct TimeSeriesTracker {
    points: Arc<Mutex<Vec<TimeSeriesPoint>>>,
    last_metrics: Arc<Mutex<Option<TestMetrics>>>,
    start_time: Arc<Mutex<chrono::DateTime<Utc>>>,
}

impl TimeSeriesTracker {
    /// Create a new time series tracker
    pub fn new() -> Self {
        Self {
            points: Arc::new(Mutex::new(Vec::new())),
            last_metrics: Arc::new(Mutex::new(None)),
            start_time: Arc::new(Mutex::new(Utc::now())),
        }
    }

    /// Add a new data point from current metrics
    pub async fn add_point(&self, metrics: &TestMetrics) {
        let start_time = *self.start_time.lock().await;
        let elapsed = (Utc::now() - start_time).num_seconds() as f64;
        
        let prev_metrics = self.last_metrics.lock().await.take();
        let point = TimeSeriesPoint::from_metrics(metrics, prev_metrics.as_ref(), elapsed);
        
        let mut points = self.points.lock().await;
        points.push(point);
        
        *self.last_metrics.lock().await = Some(metrics.clone());
    }
    
    /// Get all time series points
    pub async fn get_points(&self) -> Vec<TimeSeriesPoint> {
        let points = self.points.lock().await;
        points.clone()
    }
    
    /// Reset the tracker for a new test
    pub async fn reset(&self) {
        let mut points = self.points.lock().await;
        points.clear();
        *self.last_metrics.lock().await = None;
        *self.start_time.lock().await = Utc::now();
    }
} 