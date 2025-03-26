use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use futures::stream::{StreamExt};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use anyhow::Error;

use crate::model::state::AppState;
use crate::model::test::{
    TestType, TestStatus, ApiTestConfig, TestMetrics,
    ApiTest, // Use ApiTest instead of ApiTestRequest
    ApiRequestResult // Import ApiRequestResult directly from model::test
};
// Remove direct http::client import for ApiRequestResult
use crate::http::client::{create_optimized_client, send_api_request};
use crate::controller::test_common::TestContext;

// Helper struct to accumulate results incrementally
#[derive(Default, Debug, Clone)]
struct IncrementalApiMetrics {
    total_duration: Duration,
    status_codes: HashMap<u16, u32>,
    successful_requests: u32,
    failed_requests: u32,
    requests_completed: u32,
}

impl IncrementalApiMetrics {
    // Update accumulators based on a single request result
    fn update(&mut self, result: &Result<ApiRequestResult, Error>, expected_status: Option<u16>) {
        self.requests_completed += 1;
        match result {
            Ok(res) => {
                let mut is_success = true;
                // Use the expected_status from the ApiTest struct if available
                // Note: send_api_request needs the ApiTest struct which has expected_status
                // Let's assume expected_status is passed correctly for now.
                if let Some(expected) = expected_status {
                    if res.status != expected {
                        self.failed_requests += 1;
                        is_success = false;
                    }
                }
                if is_success {
                    self.successful_requests += 1;
                    self.total_duration += res.duration;
                }
                *self.status_codes.entry(res.status).or_insert(0) += 1;
            }
            Err(_) => {
                self.failed_requests += 1;
            }
        }
    }

    // Calculate TestMetrics based on accumulated data
    fn calculate_final_metrics(&self, total_tests: usize) -> TestMetrics {
        let avg_response_time = if self.successful_requests > 0 {
            self.total_duration.as_secs_f64() * 1000.0 / self.successful_requests as f64
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
            total_requests: total_tests as u32,
            average_response_time: avg_response_time,
            min_response_time: 0.0,
            max_response_time: 0.0,
            error_rate,
            requests_per_second: rps,
            status_codes: self.status_codes.clone(),
        }
    }
}


/// Start an API test (Refactored for Concurrency with Incremental Updates)
pub async fn start_api_test(
    State(state): State<Arc<AppState>>,
    Json(config): Json<ApiTestConfig>, // Contains Vec<ApiTest>
) -> impl IntoResponse {
    let (context, response) = match TestContext::new(state, TestType::Api).await {
        Ok((context, response)) => (context, response),
        Err(response) => return response,
    };

    let total_tests = config.tests.len();
    if total_tests == 0 {
        context.complete_test(TestMetrics::default(), Some("No tests configured".to_string())).await;
        return response;
    }

    let context = Arc::new(context);
    let tests_to_run = config.tests.clone(); // This is Vec<ApiTest>

    tokio::spawn(async move {
        let client = create_optimized_client();
        let total_tests_usize = total_tests;
        let update_interval = Duration::from_millis(500);

        let final_accumulator = futures::stream::iter(tests_to_run)
            .map(|test: ApiTest| { // Use ApiTest here
                let client = client.clone();
                async move {
                    // Get expected_status from the ApiTest struct
                    // Note: send_api_request needs adjustment if it expects ApiTestRequest
                    // Assuming send_api_request takes &ApiTest now
                    let expected_status = Some(test.expected_status); // ApiTest has non-optional expected_status
                    let result = send_api_request(&client, &test).await; // Pass ApiTest
                    (result, expected_status)
                }
            })
            .buffer_unordered(100)
            .fold( (IncrementalApiMetrics::default(), 0usize, Instant::now()),
                |mut acc: (IncrementalApiMetrics, usize, Instant), (result, expected_status)| {
                    let context = Arc::clone(&context);
                    async move {
                        let (metrics_acc, completed_count, last_update) = &mut acc;

                        metrics_acc.update(&result, expected_status);
                        *completed_count += 1;

                        let progress = (*completed_count as f32 / total_tests_usize as f32) * 100.0;

                        // Remove the incorrect type annotation in the pattern
                        let request_error_msg = match &result {
                            Ok(res) => { // No type annotation needed here
                                if let Some(expected) = expected_status {
                                    if res.status != expected {
                                        Some(format!("Req success, but status {} != expected {}", res.status, expected))
                                    } else { None }
                                } else { None } // Should not happen if expected_status is mandatory in ApiTest
                            },
                            Err(e) => Some(format!("Request failed: {}", e)),
                        };

                        let now = Instant::now();
                        if request_error_msg.is_some() || now.duration_since(*last_update) >= update_interval || *completed_count == total_tests_usize {
                            let intermediate_metrics = metrics_acc.calculate_final_metrics(total_tests_usize);
                            context.send_update(
                                TestStatus::Running,
                                progress,
                                Some(intermediate_metrics),
                                request_error_msg.clone(),
                            ).await;
                            *last_update = now;
                        }
                        acc
                    }
            }).await;

        let (final_metrics_data, _completed_count, _) = final_accumulator;
        let final_metrics = final_metrics_data.calculate_final_metrics(total_tests_usize);

        let final_error = if final_metrics_data.failed_requests > 0 {
            Some(format!("{} requests failed or had unexpected status", final_metrics_data.failed_requests))
        } else {
            None
        };

        context.complete_test(final_metrics, final_error).await;
    });

    response
}