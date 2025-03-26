use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::mpsc;
use anyhow::Error; // Import anyhow::Error

use crate::model::state::AppState;
use crate::model::test::{
    TestType, TestStatus, LoadTestConfig, TestConfig, TestMetrics, RequestResult, TimeSeriesPoint // Import RequestResult and TimeSeriesPoint here
};
// Remove RequestResult from http::client import
use crate::http::client::{create_optimized_client, load_test};
use crate::controller::test_common::TestContext;
// Removed: use std::error::Error;

// Helper struct to accumulate load test results incrementally
#[derive(Default, Debug, Clone)]
struct IncrementalLoadMetrics {
    total_duration: Duration,
    status_codes: HashMap<u16, u32>,
    successful_requests: u32,
    failed_requests: u32,
    requests_completed: u32,
    min_response_time: f64,
    max_response_time: f64,
    response_time_sum: f64,
}

impl IncrementalLoadMetrics {
    fn new() -> Self {
        Self {
            min_response_time: f64::MAX,
            ..Default::default()
        }
    }

    // Update accumulators based on a single request result (now Result<RequestResult, anyhow::Error>)
    fn update(&mut self, result: &Result<RequestResult, Error>) { // Use anyhow::Error
        self.requests_completed += 1;
        match result {
            Ok(res) => {
                self.successful_requests += 1;
                self.total_duration += res.duration;
                *self.status_codes.entry(res.status).or_insert(0) += 1;

                let duration_ms = res.duration.as_secs_f64() * 1000.0;
                self.response_time_sum += duration_ms;
                if duration_ms < self.min_response_time {
                    self.min_response_time = duration_ms;
                }
                if duration_ms > self.max_response_time {
                    self.max_response_time = duration_ms;
                }
            }
            Err(_) => {
                self.failed_requests += 1;
            }
        }
    }

    // Calculate TestMetrics based on accumulated data
    fn calculate_metrics(&self, total_planned_requests: u32) -> TestMetrics {
        let avg_response_time = if self.successful_requests > 0 {
            self.response_time_sum / self.successful_requests as f64
        } else {
            0.0
        };
        let rps = if !self.total_duration.is_zero() {
            self.successful_requests as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        };
        let error_rate = if self.requests_completed > 0 {
            (self.failed_requests as f64 / self.requests_completed as f64) * 100.0
        } else {
            0.0
        };

        TestMetrics {
            requests_completed: self.requests_completed,
            total_requests: total_planned_requests,
            average_response_time: avg_response_time,
            min_response_time: if self.min_response_time == f64::MAX { 0.0 } else { self.min_response_time },
            max_response_time: self.max_response_time,
            error_rate,
            requests_per_second: rps,
            status_codes: self.status_codes.clone(),
        }
    }

    // Calculate TimeSeriesPoint for chart data
    fn calculate_time_series_point(&self) -> TimeSeriesPoint {
        let rps = if !self.total_duration.is_zero() {
            self.successful_requests as f64 / self.total_duration.as_secs_f64()
        } else {
            0.0
        };
        let avg_response_time = if self.successful_requests > 0 {
            self.response_time_sum / self.successful_requests as f64
        } else {
            0.0
        };
        let error_rate = if self.requests_completed > 0 {
            (self.failed_requests as f64 / self.requests_completed as f64) * 100.0
        } else {
            0.0
        };

        TimeSeriesPoint {
            timestamp: chrono::Utc::now().timestamp_millis(),
            requests_per_second: rps,
            average_response_time: avg_response_time,
            error_rate,
        }
    }
}


/// Start a load test (Refactored for Channel Aggregation)
pub async fn start_load_test(
    State(state): State<Arc<AppState>>,
    Json(config): Json<LoadTestConfig>,
) -> impl IntoResponse {
    let (context, response) = match TestContext::new(state, TestType::Load).await {
        Ok((context, response)) => (context, response),
        Err(response) => return response,
    };

    let test_config = TestConfig {
        target_url: config.target_url,
        concurrent_users: config.concurrent_users.unwrap_or(10),
        duration_secs: 0,
        num_requests: config.num_requests,
    };

    if test_config.num_requests == 0 {
        context.complete_test(TestMetrics::default(), Some("Number of requests must be greater than 0 for load test".to_string())).await;
        return response;
    }

    let context = Arc::new(context);
    let total_planned_requests = test_config.num_requests;

    tokio::spawn(async move {
        let client = create_optimized_client();
        let (result_tx, mut result_rx) = mpsc::channel::<Result<RequestResult, Error>>(1024); // Use anyhow::Error

        let is_finished = Arc::new(AtomicBool::new(false));
        let context_clone = Arc::clone(&context);

        // Spawn Aggregator Task
        let aggregator_handle = tokio::spawn(async move {
            let mut metrics_agg = IncrementalLoadMetrics::new();
            let update_interval = Duration::from_millis(100);
            let mut last_update_time = Instant::now();
            let mut received_count = 0u32;

            tracing::info!("Aggregator task started for load test {}", context_clone.test_id());

            while let Some(result) = result_rx.recv().await {
                metrics_agg.update(&result);
                received_count += 1;

                let progress = (received_count as f32 / total_planned_requests as f32) * 100.0;
                let now = Instant::now();

                if now.duration_since(last_update_time) >= update_interval || received_count == total_planned_requests {
                    let intermediate_metrics = metrics_agg.calculate_metrics(total_planned_requests);
                    let error_string = result.err().map(|e| format!("{:?}", e));

                    // Send both types of updates
                    context_clone.send_update(
                        TestStatus::Running,
                        progress.min(100.0),
                        Some(intermediate_metrics.clone()),
                        error_string,
                    ).await;

                    // Update time series data with TestMetrics
                    if let Err(e) = context_clone.update_time_series(&intermediate_metrics).await {
                        tracing::warn!("Failed to update time series: {}", e);
                    }

                    last_update_time = now;
                }
            }

            tracing::info!("Aggregator channel closed for load test {}. Calculating final metrics.", context_clone.test_id());
            let final_metrics = metrics_agg.calculate_metrics(total_planned_requests);
            let final_error = if metrics_agg.failed_requests > 0 {
                Some(format!("{} requests failed", metrics_agg.failed_requests))
            } else {
                None
            };

            // Send final update
            context_clone.send_update(
                TestStatus::Completed,
                100.0,
                Some(final_metrics.clone()),
                final_error.clone(),
            ).await;

            // Update time series one last time
            if let Err(e) = context_clone.update_time_series(&final_metrics).await {
                tracing::warn!("Failed to update final time series: {}", e);
            }

            context_clone.complete_test(final_metrics, final_error).await;
            tracing::info!("Aggregator task finished for load test {}.", context_clone.test_id());
        });

        // Start the load test execution
        if let Err(e) = load_test(&client, &test_config, result_tx, Arc::clone(&is_finished)).await {
            tracing::error!("Failed to start load_test function for test {}: {}", context.test_id(), e);
            is_finished.store(true, Ordering::SeqCst);
            aggregator_handle.abort();

            let error_msg = format!("Failed to start load test: {}", e);
            context.send_update(
                TestStatus::Error,
                0.0,
                None,
                Some(error_msg.clone()),
            ).await;
            context.complete_test(TestMetrics::default(), Some(error_msg)).await;
        }
    });

    response
}