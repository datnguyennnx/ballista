use serde::{Deserialize, Serialize};
use chrono::Utc;
use std::collections::HashMap;
use std::time::Duration;

use crate::model::test::TestMetrics;

/// Time series data point that matches the frontend's TimeSeriesPoint interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: i64,
    pub responseTime: f64,
    pub requestsPerSecond: f64,
    pub errorRate: f64,
    pub concurrentUsers: Option<f64>,
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

        // Calculate error rate (errors / total requests)
        let error_rate = if metrics.requests_completed > 0 {
            (metrics.errors as f64 / metrics.requests_completed as f64) * 100.0
        } else {
            0.0
        };

        // Calculate concurrent users (simulation based on current throughput)
        // This is a simple estimation - in a real system you might want to track actual concurrent users
        let concurrent_users = Some(rps * 2.5);

        Self {
            timestamp: Utc::now().timestamp_millis(),
            responseTime: metrics.avg_response_time,
            requestsPerSecond: rps,
            errorRate: error_rate,
            concurrentUsers: concurrent_users,
        }
    }
}

/// Helper struct to track and generate time series data
pub struct TimeSeriesTracker {
    pub points: Vec<TimeSeriesPoint>,
    pub last_metrics: Option<TestMetrics>,
    pub last_update_time: std::time::Instant,
    pub test_start_time: std::time::Instant,
}

impl TimeSeriesTracker {
    /// Create a new time series tracker
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            last_metrics: None,
            last_update_time: std::time::Instant::now(),
            test_start_time: std::time::Instant::now(),
        }
    }

    /// Add a new data point from current metrics
    pub fn add_point(&mut self, metrics: &TestMetrics) {
        let now = std::time::Instant::now();
        let elapsed_since_last = now.duration_since(self.last_update_time).as_secs_f64();
        let _elapsed_total = now.duration_since(self.test_start_time).as_secs_f64();
        
        let point = TimeSeriesPoint::from_metrics(
            metrics, 
            self.last_metrics.as_ref(),
            elapsed_since_last,
        );
        
        self.points.push(point);
        self.last_metrics = Some(metrics.clone());
        self.last_update_time = now;
        
        // Keep only the last 100 points to avoid excessive memory usage
        if self.points.len() > 100 {
            self.points.remove(0);
        }
    }
    
    /// Get all time series points
    pub fn get_points(&self) -> Vec<TimeSeriesPoint> {
        self.points.clone()
    }
    
    /// Get the latest time series point
    pub fn get_latest_point(&self) -> Option<TimeSeriesPoint> {
        self.points.last().cloned()
    }
    
    /// Reset the tracker for a new test
    pub fn reset(&mut self) {
        self.points.clear();
        self.last_metrics = None;
        self.last_update_time = std::time::Instant::now();
        self.test_start_time = std::time::Instant::now();
    }
} 