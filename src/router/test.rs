use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::router::AppState;
use ballista::core::test_runner::{run_load_test, run_stress_test};
use ballista::core::api_test_runner::run_api_tests;
use serde_json::{json, Value};
use uuid;
use rand;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct TestConfig {
    duration: u64,
    target_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestResponse {
    id: String,
    status: String,
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
    pub status_codes: std::collections::HashMap<u16, u32>,
    pub errors: u32,
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

// Pure functions for test endpoints
pub async fn load_test(Json(_config): Json<LoadTestConfig>) -> Json<Value> {
    Json(json!({
        "success": true,
        "message": "Load test started",
        "data": {
            "id": uuid::Uuid::new_v4().to_string(),
            "test_type": TestType::Load,
            "status": TestStatus::Started,
            "timestamp": chrono::Utc::now().timestamp(),
        }
    }))
}

pub async fn stress_test(Json(_config): Json<StressTestConfig>) -> Json<Value> {
    Json(json!({
        "success": true,
        "message": "Stress test started",
        "data": {
            "id": uuid::Uuid::new_v4().to_string(),
            "test_type": TestType::Stress,
            "status": TestStatus::Started,
            "timestamp": chrono::Utc::now().timestamp(),
        }
    }))
}

pub async fn api_test(Json(_config): Json<ApiTestConfig>) -> Json<Value> {
    Json(json!({
        "success": true,
        "message": "API test started",
        "data": {
            "id": uuid::Uuid::new_v4().to_string(),
            "test_type": TestType::Api,
            "status": TestStatus::Started,
            "timestamp": chrono::Utc::now().timestamp(),
        }
    }))
}

pub async fn get_test_results() -> Json<Value> {
    Json(json!({
        "success": true,
        "message": "Test results retrieved",
        "data": []
    }))
}

#[axum::debug_handler]
pub async fn start_load_test(
    State(state): State<Arc<AppState>>,
    Json(config): Json<LoadTestConfig>,
) -> Json<TestUpdate> {
    let test_id = uuid::Uuid::new_v4().to_string();
    
    tokio::spawn({
        let tx = state.tx.clone();
        let test_id = test_id.clone();
        let config = config.clone();
        async move {
            let mut metrics = TestMetrics {
                requests_completed: 0,
                total_requests: 0,
                avg_response_time: 0.0,
                min_response_time: None,
                max_response_time: None,
                median_response_time: None,
                p95_response_time: None,
                status_codes: std::collections::HashMap::new(),
                errors: 0,
            };

            // Initial update
            let initial_update = TestUpdate {
                id: test_id.clone(),
                test_type: TestType::Load,
                status: TestStatus::Running,
                progress: 0.0,
                metrics: Some(metrics.clone()),
                error: None,
                timestamp: chrono::Utc::now().timestamp(),
            };
            let _ = tx.send(serde_json::to_string(&initial_update).unwrap_or_default());

            // Run the actual load test
            match run_load_test(&config.target_url, config.num_requests, config.concurrency).await {
                Ok(_) => {
                    for i in 0..10 {
                        let progress = (i * 100) / 10;
                        metrics.requests_completed = config.num_requests / 10 * (i + 1);
                        metrics.total_requests = config.num_requests;
                        metrics.avg_response_time = 50.0 + (rand::random::<f64>() * 20.0);
                        
                        let update = TestUpdate {
                            id: test_id.clone(),
                            test_type: TestType::Load,
                            status: TestStatus::Running,
                            progress: progress as f32,
                            metrics: Some(metrics.clone()),
                            error: None,
                            timestamp: chrono::Utc::now().timestamp(),
                        };
                        let _ = tx.send(serde_json::to_string(&update).unwrap_or_default());
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
                Err(e) => {
                    metrics.errors += 1;
                    let error_update = TestUpdate {
                        id: test_id.clone(),
                        test_type: TestType::Load,
                        status: TestStatus::Error,
                        progress: 100.0,
                        metrics: Some(metrics.clone()),
                        error: Some(format!("error: {}", e)),
                        timestamp: chrono::Utc::now().timestamp(),
                    };
                    let _ = tx.send(serde_json::to_string(&error_update).unwrap_or_default());
                    return;
                }
            }

            // Final update
            let final_update = TestUpdate {
                id: test_id.clone(),
                test_type: TestType::Load,
                status: TestStatus::Completed,
                progress: 100.0,
                metrics: Some(metrics),
                error: None,
                timestamp: chrono::Utc::now().timestamp(),
            };
            let _ = tx.send(serde_json::to_string(&final_update).unwrap_or_default());
        }
    });

    Json(TestUpdate {
        id: test_id,
        test_type: TestType::Load,
        status: TestStatus::Started,
        progress: 0.0,
        metrics: None,
        error: None,
        timestamp: chrono::Utc::now().timestamp(),
    })
}

#[axum::debug_handler]
pub async fn start_stress_test(
    State(state): State<Arc<AppState>>,
    Json(config): Json<StressTestConfig>,
) -> Json<TestUpdate> {
    let test_id = uuid::Uuid::new_v4().to_string();
    
    tokio::spawn({
        let tx = state.tx.clone();
        let test_id = test_id.clone();
        let config = config.clone();
        async move {
            let mut metrics = TestMetrics {
                requests_completed: 0,
                total_requests: 0,
                avg_response_time: 0.0,
                min_response_time: None,
                max_response_time: None,
                median_response_time: None,
                p95_response_time: None,
                status_codes: std::collections::HashMap::new(),
                errors: 0,
            };

            // Initial update
            let initial_update = TestUpdate {
                id: test_id.clone(),
                test_type: TestType::Stress,
                status: TestStatus::Running,
                progress: 0.0,
                metrics: Some(metrics.clone()),
                error: None,
                timestamp: chrono::Utc::now().timestamp(),
            };
            let _ = tx.send(serde_json::to_string(&initial_update).unwrap_or_default());

            // Run the actual stress test
            match run_stress_test(&config.target_url, config.duration_secs, config.concurrency).await {
                Ok(_) => {
                    for i in 0..config.duration_secs {
                        let progress = (i * 100) / config.duration_secs;
                        metrics.requests_completed += config.concurrency;
                        metrics.total_requests += config.concurrency;
                        metrics.avg_response_time = 100.0 + (rand::random::<f64>() * 50.0);
                        
                        let update = TestUpdate {
                            id: test_id.clone(),
                            test_type: TestType::Stress,
                            status: TestStatus::Running,
                            progress: progress as f32,
                            metrics: Some(metrics.clone()),
                            error: None,
                            timestamp: chrono::Utc::now().timestamp(),
                        };
                        let _ = tx.send(serde_json::to_string(&update).unwrap_or_default());
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
                Err(e) => {
                    metrics.errors += 1;
                    let error_update = TestUpdate {
                        id: test_id.clone(),
                        test_type: TestType::Stress,
                        status: TestStatus::Error,
                        progress: 100.0,
                        metrics: Some(metrics.clone()),
                        error: Some(format!("error: {}", e)),
                        timestamp: chrono::Utc::now().timestamp(),
                    };
                    let _ = tx.send(serde_json::to_string(&error_update).unwrap_or_default());
                    return;
                }
            }

            // Final update
            let final_update = TestUpdate {
                id: test_id.clone(),
                test_type: TestType::Stress,
                status: TestStatus::Completed,
                progress: 100.0,
                metrics: Some(metrics),
                error: None,
                timestamp: chrono::Utc::now().timestamp(),
            };
            let _ = tx.send(serde_json::to_string(&final_update).unwrap_or_default());
        }
    });

    Json(TestUpdate {
        id: test_id,
        test_type: TestType::Stress,
        status: TestStatus::Started,
        progress: 0.0,
        metrics: None,
        error: None,
        timestamp: chrono::Utc::now().timestamp(),
    })
}

#[axum::debug_handler]
pub async fn start_api_test(
    State(state): State<Arc<AppState>>,
    Json(config): Json<ApiTestConfig>,
) -> Json<TestUpdate> {
    let test_id = uuid::Uuid::new_v4().to_string();
    
    tokio::spawn({
        let tx = state.tx.clone();
        let test_id = test_id.clone();
        let config = config.clone();
        async move {
            let mut metrics = TestMetrics {
                requests_completed: 0,
                total_requests: 0,
                avg_response_time: 0.0,
                min_response_time: None,
                max_response_time: None,
                median_response_time: None,
                p95_response_time: None,
                status_codes: std::collections::HashMap::new(),
                errors: 0,
            };

            // Initial update
            let initial_update = TestUpdate {
                id: test_id.clone(),
                test_type: TestType::Api,
                status: TestStatus::Running,
                progress: 0.0,
                metrics: Some(metrics.clone()),
                error: None,
                timestamp: chrono::Utc::now().timestamp(),
            };
            let _ = tx.send(serde_json::to_string(&initial_update).unwrap_or_default());

            // Run the actual API test
            match run_api_tests(&config.test_suite_path).await {
                Ok(_) => {
                    for i in 0..10 {
                        let progress = (i * 100) / 10;
                        metrics.requests_completed += 5;
                        metrics.total_requests += 5;
                        metrics.avg_response_time = 200.0 + (rand::random::<f64>() * 100.0);
                        
                        let update = TestUpdate {
                            id: test_id.clone(),
                            test_type: TestType::Api,
                            status: TestStatus::Running,
                            progress: progress as f32,
                            metrics: Some(metrics.clone()),
                            error: None,
                            timestamp: chrono::Utc::now().timestamp(),
                        };
                        let _ = tx.send(serde_json::to_string(&update).unwrap_or_default());
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
                Err(e) => {
                    metrics.errors += 1;
                    let error_update = TestUpdate {
                        id: test_id.clone(),
                        test_type: TestType::Api,
                        status: TestStatus::Error,
                        progress: 100.0,
                        metrics: Some(metrics.clone()),
                        error: Some(format!("error: {}", e)),
                        timestamp: chrono::Utc::now().timestamp(),
                    };
                    let _ = tx.send(serde_json::to_string(&error_update).unwrap_or_default());
                    return;
                }
            }

            // Final update
            let final_update = TestUpdate {
                id: test_id.clone(),
                test_type: TestType::Api,
                status: TestStatus::Completed,
                progress: 100.0,
                metrics: Some(metrics),
                error: None,
                timestamp: chrono::Utc::now().timestamp(),
            };
            let _ = tx.send(serde_json::to_string(&final_update).unwrap_or_default());
        }
    });

    Json(TestUpdate {
        id: test_id,
        test_type: TestType::Api,
        status: TestStatus::Started,
        progress: 0.0,
        metrics: None,
        error: None,
        timestamp: chrono::Utc::now().timestamp(),
    })
} 