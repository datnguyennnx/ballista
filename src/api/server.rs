use axum::{
    routing::{get, post},
    Router,
    extract::{Json, State},
    http::header,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::core::app::{AppState, TestResult, TestType, TestStatus};
use crate::core::test_runner::{run_load_test, run_stress_test};
use crate::core::api_test_runner::run_api_tests;
use chrono::Utc;
// Correct imports for Axum 0.7 middleware
use axum::middleware::map_response;
use axum::response::Response;

// Generic API Response type
#[derive(Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

// Request DTOs
#[derive(Deserialize)]
pub struct LoadTestRequest {
    url: String,
    requests: u32,
    concurrency: u32,
}

#[derive(Deserialize)]
pub struct StressTestRequest {
    sitemap: String,
    duration: u64,
    concurrency: u32,
}

#[derive(Deserialize)]
pub struct ApiTestRequest {
    path: String,
}

// Pure function to add CORS headers
async fn add_cors_headers(response: Response) -> Response {
    let mut response = response;
    let headers = response.headers_mut();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        header::HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        header::HeaderValue::from_static("Content-Type, Authorization"),
    );
    response
}

// Pure function to create test result
fn create_test_result(test_type: TestType, status: TestStatus, details: String) -> TestResult {
    TestResult {
        test_type,
        status,
        details,
        timestamp: Utc::now(),
    }
}

// Pure function to create API response
fn create_api_response<T>(success: bool, message: String, data: Option<T>) -> ApiResponse<T> {
    ApiResponse {
        success,
        message,
        data,
    }
}

// Handler functions
async fn health_check() -> Json<ApiResponse<()>> {
    Json(create_api_response(
        true,
        "API is running".to_string(),
        None,
    ))
}

async fn get_test_results(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<Vec<TestResult>>> {
    let results = state.get_test_results();
    Json(create_api_response(
        true,
        "Test results retrieved".to_string(),
        Some(results),
    ))
}

async fn run_load_test_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoadTestRequest>,
) -> Json<ApiResponse<TestResult>> {
    let initial_result = create_test_result(
        TestType::Load,
        TestStatus::InProgress,
        format!("Running load test for {}", payload.url),
    );
    state.add_test_result(initial_result.clone());

    tokio::spawn({
        let state = state.clone();
        async move {
            match run_load_test(&payload.url, payload.requests, payload.concurrency).await {
                Ok(_) => {
                    let result = create_test_result(
                        TestType::Load,
                        TestStatus::Success,
                        "Load test completed successfully".to_string(),
                    );
                    state.add_test_result(result);
                }
                Err(e) => {
                    let result = create_test_result(
                        TestType::Load,
                        TestStatus::Failure,
                        format!("Load test failed: {}", e),
                    );
                    state.add_test_result(result);
                }
            }
        }
    });

    Json(create_api_response(
        true,
        "Load test started".to_string(),
        Some(initial_result),
    ))
}

// Similar handler for stress test
async fn run_stress_test_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<StressTestRequest>,
) -> Json<ApiResponse<TestResult>> {
    let initial_result = create_test_result(
        TestType::Stress,
        TestStatus::InProgress,
        format!("Running stress test for sitemap {}", payload.sitemap),
    );
    state.add_test_result(initial_result.clone());

    tokio::spawn({
        let state = state.clone();
        async move {
            match run_stress_test(&payload.sitemap, payload.duration.into(), payload.concurrency).await {
                Ok(_) => {
                    let result = create_test_result(
                        TestType::Stress,
                        TestStatus::Success,
                        "Stress test completed successfully".to_string(),
                    );
                    state.add_test_result(result);
                }
                Err(e) => {
                    let result = create_test_result(
                        TestType::Stress,
                        TestStatus::Failure,
                        format!("Stress test failed: {}", e),
                    );
                    state.add_test_result(result);
                }
            }
        }
    });

    Json(create_api_response(
        true,
        "Stress test started".to_string(),
        Some(initial_result),
    ))
}

// API test handler
async fn run_api_test_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ApiTestRequest>,
) -> Json<ApiResponse<TestResult>> {
    let initial_result = create_test_result(
        TestType::Api,
        TestStatus::InProgress,
        format!("Running API test from {}", payload.path),
    );
    state.add_test_result(initial_result.clone());

    tokio::spawn({
        let state = state.clone();
        async move {
            match run_api_tests(&payload.path).await {
                Ok(_) => {
                    let result = create_test_result(
                        TestType::Api,
                        TestStatus::Success,
                        "API test completed successfully".to_string(),
                    );
                    state.add_test_result(result);
                }
                Err(e) => {
                    let result = create_test_result(
                        TestType::Api,
                        TestStatus::Failure,
                        format!("API test failed: {}", e),
                    );
                    state.add_test_result(result);
                }
            }
        }
    });

    Json(create_api_response(
        true,
        "API test started".to_string(),
        Some(initial_result),
    ))
}

pub async fn create_api_server(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/tests", get(get_test_results))
        .route("/api/load-test", post(run_load_test_handler))
        .route("/api/stress-test", post(run_stress_test_handler))
        .route("/api/api-test", post(run_api_test_handler))
        .layer(map_response(add_cors_headers))
        .with_state(state)
} 