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
    TestType, TestStatus, StressTestConfig, TestConfig, TestMetrics, RequestResult // Import RequestResult here
};
// Remove RequestResult from http::client import
use crate::http::client::{create_optimized_client, stress_test};
use crate::controller::test_common::TestContext;
// Removed: use std::error::Error;

// Helper struct to accumulate stress test results incrementally
#[derive(Default, Debug, Clone)]
struct IncrementalStressMetrics {
    total_duration: Duration,
    status_codes: HashMap<u16, u32>,
    successful_requests: u32,
    failed_requests: u32,
    requests_completed: u32,
    min_response_time: f64,
    max_response_time: f64,
    response_time_sum: f64,
}

impl IncrementalStressMetrics {
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
    fn calculate_metrics(&self) -> TestMetrics {
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
            total_requests: self.requests_completed, // For stress test, total = completed
            average_response_time: avg_response_time,
            min_response_time: if self.min_response_time == f64::MAX { 0.0 } else { self.min_response_time },
            max_response_time: self.max_response_time,
            error_rate,
            requests_per_second: rps,
            status_codes: self.status_codes.clone(),
        }
    }
}


/// Start a stress test (Refactored for Channel Aggregation)
pub async fn start_stress_test(
    State(state): State<Arc<AppState>>,
    Json(config): Json<StressTestConfig>,
) -> impl IntoResponse {
    let (context, response) = match TestContext::new(state, TestType::Stress).await {
        Ok((context, response)) => (context, response),
        Err(response) => return response,
    };

    let test_config = TestConfig {
        target_url: config.target_url,
        concurrent_users: config.concurrent_users,
        duration_secs: config.duration_secs,
        num_requests: 0,
    };

     if test_config.duration_secs == 0 {
         context.complete_test(TestMetrics::default(), Some("Duration must be greater than 0 for stress test".to_string())).await;
         return response;
     }

    let context = Arc::new(context);
    let test_duration = Duration::from_secs(test_config.duration_secs as u64);

    tokio::spawn(async move {
        let client = create_optimized_client();
        // Channel now sends Result<RequestResult, anyhow::Error>
        let (result_tx, mut result_rx) = mpsc::channel::<Result<RequestResult, Error>>(1024);

        let is_finished = Arc::new(AtomicBool::new(false));
        let context_clone = Arc::clone(&context);
        let start_time = Instant::now();

        // --- Spawn Aggregator Task ---
        let aggregator_handle = tokio::spawn(async move {
            let mut metrics_agg = IncrementalStressMetrics::new();
            let update_interval = Duration::from_millis(500);
            let mut last_update_time = Instant::now();

            tracing::info!("Aggregator task started for stress test {}", context_clone.test_id());

            while let Some(result) = result_rx.recv().await {
                metrics_agg.update(&result);

                let elapsed = start_time.elapsed();
                let progress = (elapsed.as_secs_f64() / test_duration.as_secs_f64() * 100.0).min(100.0);

                let now = Instant::now();
                 if now.duration_since(last_update_time) >= update_interval {
                    let intermediate_metrics = metrics_agg.calculate_metrics();
                    // Correctly format the anyhow::Error to String for send_update
                    let error_string = result.err().map(|e| format!("{:?}", e)); // Use Debug format

                    context_clone.send_update(
                        TestStatus::Running,
                        progress as f32,
                        Some(intermediate_metrics),
                        error_string, // Pass formatted error string
                    ).await;
                    last_update_time = now;
                }
            }
            tracing::info!("Aggregator channel closed for stress test {}. Calculating final metrics.", context_clone.test_id());
            let final_metrics = metrics_agg.calculate_metrics();
            let final_error = if metrics_agg.failed_requests > 0 {
                 Some(format!("{} requests failed", metrics_agg.failed_requests))
            } else {
                 None
            };
            context_clone.send_update(TestStatus::Running, 100.0, Some(final_metrics.clone()), final_error.clone()).await;
            context_clone.complete_test(final_metrics, final_error).await;
            tracing::info!("Aggregator task finished for stress test {}.", context_clone.test_id());
        });

        // --- Start the stress test execution ---
        if let Err(e) = stress_test(&client, &test_config, result_tx, Arc::clone(&is_finished)).await {
             tracing::error!("Failed during stress_test function for test {}: {}", context.test_id(), e);
            is_finished.store(true, Ordering::SeqCst);
            aggregator_handle.abort();

            let error_msg = format!("Stress test failed during execution: {}", e);
            let current_progress = (start_time.elapsed().as_secs_f64() / test_duration.as_secs_f64() * 100.0).min(100.0) as f32;
            context.send_update(
                TestStatus::Error,
                current_progress,
                None,
                Some(error_msg.clone()),
            ).await;
            context.complete_test(TestMetrics::default(), Some(error_msg)).await;
        } else {
             tracing::info!("stress_test function finished successfully for test {}.", context.test_id());
        }
    });

    response
}