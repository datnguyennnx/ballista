use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::time::Duration;

use crate::model::state::AppState;
use crate::model::test::{
    LoadTestConfig, StressTestConfig, ApiTestConfig, 
    TestType, TestStatus, create_test_result, create_test_update,
    create_test_config_from_load, create_test_config_from_stress,
};
use crate::view::response::create_api_response;
use crate::http::client::{send_request, send_api_request};

/// Get all test results
pub async fn get_all_test_results(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let results = state.get_all_test_results().await;
    Json(create_api_response(
        true,
        "Test results retrieved".to_string(),
        Some(results),
    ))
}

/// Start a load test
pub async fn start_load_test(
    State(state): State<Arc<AppState>>,
    Json(config): Json<LoadTestConfig>,
) -> impl IntoResponse {
    // Check if a test is already running
    if state.is_running.load(Ordering::SeqCst) {
        return Json(create_api_response(
            false,
            "Another test is already running".to_string(),
            None::<String>,
        ));
    }
    
    // Create a unique test ID
    let test_id = state.generate_test_id();
    
    // Mark test as running
    state.is_running.store(true, Ordering::SeqCst);
    
    // Create initial test result
    let initial_result = create_test_result(
        test_id.clone(),
        TestType::Load,
        TestStatus::Started,
        None,
        None,
    );
    
    // Add initial result to state
    state.add_test_result(initial_result.clone()).await;
    
    // Send initial update
    let initial_update = create_test_update(
        test_id.clone(),
        TestType::Load,
        TestStatus::Started,
        0.0,
        None,
        None,
    );
    let _ = state.send_test_update(initial_update).await;
    
    // Reset time series tracker for the new test
    state.reset_time_series();
    
    // Run the test in the background
    tokio::spawn({
        let state = Arc::clone(&state);
        let test_id = test_id.clone();
        let config = config.clone();
        
        async move {
            // Convert to test config
            let _test_config = create_test_config_from_load(&config);
            let client = reqwest::Client::new();
            
            // Create metrics collectors
            let mut durations = Vec::new();
            let mut status_codes = std::collections::HashMap::new();
            let mut errors = 0;
            let mut requests_completed = 0;
            let total_requests = config.num_requests;
            
            // Update status to running
            let running_update = create_test_update(
                test_id.clone(),
                TestType::Load,
                TestStatus::Running,
                0.0,
                None,
                None,
            );
            let _ = state.send_test_update(running_update).await;
            
            // Run the load test
            for _ in 0..total_requests {
                match send_request(&client, &config.target_url).await {
                    Ok(result) => {
                        durations.push(result.duration);
                        *status_codes.entry(result.status).or_insert(0) += 1;
                        if result.error.is_some() {
                            errors += 1;
                        }
                    },
                    Err(_) => {
                        errors += 1;
                    }
                }
                
                requests_completed += 1;
                
                // Send progress updates periodically
                if requests_completed % 10 == 0 || requests_completed == total_requests {
                    let progress = requests_completed as f32 / total_requests as f32 * 100.0;
                    
                    // Create metrics
                    let metrics = crate::model::test::create_test_metrics(
                        requests_completed,
                        total_requests,
                        &durations,
                        status_codes.clone(),
                        errors,
                    );
                    
                    // Update time series and send real-time data
                    let _ = state.update_time_series(&metrics).await;
                    
                    // Send update
                    let update = create_test_update(
                        test_id.clone(),
                        TestType::Load,
                        TestStatus::Running,
                        progress,
                        Some(metrics),
                        None,
                    );
                    let _ = state.send_test_update(update).await;
                }
            }
            
            // Create final metrics
            let final_metrics = crate::model::test::create_test_metrics(
                requests_completed,
                total_requests,
                &durations,
                status_codes,
                errors,
            );
            
            // Update with completed status
            let final_result = create_test_result(
                test_id.clone(),
                TestType::Load,
                TestStatus::Completed,
                Some(final_metrics.clone()),
                None,
            );
            
            // Add final result
            state.add_test_result(final_result.clone()).await;
            
            // Send final update
            let final_update = create_test_update(
                test_id,
                TestType::Load,
                TestStatus::Completed,
                100.0,
                Some(final_metrics),
                None,
            );
            let _ = state.send_test_update(final_update).await;
            
            // Mark test as not running
            state.is_running.store(false, Ordering::SeqCst);
        }
    });
    
    // Return the initial result
    Json(create_api_response(
        true,
        "Load test started".to_string(),
        Some(initial_result.id),
    ))
}

/// Start a stress test
pub async fn start_stress_test(
    State(state): State<Arc<AppState>>,
    Json(config): Json<StressTestConfig>,
) -> impl IntoResponse {
    // Check if a test is already running
    if state.is_running.load(Ordering::SeqCst) {
        return Json(create_api_response(
            false,
            "Another test is already running".to_string(),
            None::<String>,
        ));
    }
    
    // Create a unique test ID
    let test_id = state.generate_test_id();
    
    // Mark test as running
    state.is_running.store(true, Ordering::SeqCst);
    
    // Create initial test result
    let initial_result = create_test_result(
        test_id.clone(),
        TestType::Stress,
        TestStatus::Started,
        None,
        None,
    );
    
    // Add initial result to state
    state.add_test_result(initial_result.clone()).await;
    
    // Send initial update
    let initial_update = create_test_update(
        test_id.clone(),
        TestType::Stress,
        TestStatus::Started,
        0.0,
        None,
        None,
    );
    let _ = state.send_test_update(initial_update).await;
    
    // Reset time series tracker for the new test
    state.reset_time_series();
    
    // Run the test in the background
    tokio::spawn({
        let state = Arc::clone(&state);
        let test_id = test_id.clone();
        let config = config.clone();
        
        async move {
            // Convert to test config
            let _test_config = create_test_config_from_stress(&config);
            let client = reqwest::Client::new();
            
            // Create metrics collectors
            let mut durations = Vec::new();
            let mut status_codes = std::collections::HashMap::new();
            let mut errors = 0;
            let mut requests_completed = 0;
            
            // Set up test duration
            let start_time = std::time::Instant::now();
            let end_time = start_time + std::time::Duration::from_secs(config.duration_secs);
            
            // Update status to running
            let running_update = create_test_update(
                test_id.clone(),
                TestType::Stress,
                TestStatus::Running,
                0.0,
                None,
                None,
            );
            let _ = state.send_test_update(running_update).await;
            
            // Run the stress test
            while std::time::Instant::now() < end_time {
                // Create a batch of requests based on concurrency
                let futures = (0..config.concurrency).map(|_| {
                    let client = client.clone();
                    let url = config.target_url.clone();
                    async move {
                        send_request(&client, &url).await
                    }
                });
                
                // Run batch and collect results
                let batch_results = futures::future::join_all(futures).await;
                
                // Process batch results
                for result in batch_results {
                    match result {
                        Ok(req_result) => {
                            durations.push(req_result.duration);
                            *status_codes.entry(req_result.status).or_insert(0) += 1;
                            if req_result.error.is_some() {
                                errors += 1;
                            }
                        },
                        Err(_) => {
                            errors += 1;
                        }
                    }
                    requests_completed += 1;
                }
                
                // Calculate progress as percentage of time elapsed
                let elapsed = start_time.elapsed();
                let total_duration = std::time::Duration::from_secs(config.duration_secs);
                let progress = (elapsed.as_secs_f32() / total_duration.as_secs_f32() * 100.0).min(100.0);
                
                // Create metrics
                let metrics = crate::model::test::create_test_metrics(
                    requests_completed,
                    0, // Unknown total for stress test
                    &durations,
                    status_codes.clone(),
                    errors,
                );
                
                // Update time series and send real-time data
                let _ = state.update_time_series(&metrics).await;
                
                // Send update
                let update = create_test_update(
                    test_id.clone(),
                    TestType::Stress,
                    TestStatus::Running,
                    progress,
                    Some(metrics),
                    None,
                );
                let _ = state.send_test_update(update).await;
                
                // Small delay to prevent CPU overload
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            
            // Create final metrics
            let final_metrics = crate::model::test::create_test_metrics(
                requests_completed,
                0, // Unknown total for stress test
                &durations,
                status_codes,
                errors,
            );
            
            // Update with completed status
            let final_result = create_test_result(
                test_id.clone(),
                TestType::Stress,
                TestStatus::Completed,
                Some(final_metrics.clone()),
                None,
            );
            
            // Add final result
            state.add_test_result(final_result.clone()).await;
            
            // Send final update
            let final_update = create_test_update(
                test_id,
                TestType::Stress,
                TestStatus::Completed,
                100.0,
                Some(final_metrics),
                None,
            );
            let _ = state.send_test_update(final_update).await;
            
            // Mark test as not running
            state.is_running.store(false, Ordering::SeqCst);
        }
    });
    
    // Return the initial result
    Json(create_api_response(
        true,
        "Stress test started".to_string(),
        Some(initial_result.id),
    ))
}

/// Start an API test
pub async fn start_api_test(
    State(state): State<Arc<AppState>>,
    Json(config): Json<ApiTestConfig>,
) -> impl IntoResponse {
    // Check if a test is already running
    if state.is_running.load(Ordering::SeqCst) {
        return Json(create_api_response(
            false,
            "Another test is already running".to_string(),
            None::<String>,
        ));
    }
    
    // Create a unique test ID
    let test_id = state.generate_test_id();
    
    // Mark test as running
    state.is_running.store(true, Ordering::SeqCst);
    
    // Create initial test result
    let initial_result = create_test_result(
        test_id.clone(),
        TestType::Api,
        TestStatus::Started,
        None,
        None,
    );
    
    // Add initial result to state
    state.add_test_result(initial_result.clone()).await;
    
    // Run the test in the background
    tokio::spawn({
        let state = Arc::clone(&state);
        let test_id = test_id.clone();
        let path = config.test_suite_path.clone();
        
        async move {
            // Update status to running
            let running_update = create_test_update(
                test_id.clone(),
                TestType::Api,
                TestStatus::Running,
                0.0,
                None,
                None,
            );
            let _ = state.send_test_update(running_update).await;
            
            // Load API tests from file
            let api_tests = match tokio::fs::read_to_string(&path).await {
                Ok(content) => match serde_json::from_str::<Vec<crate::model::test::ApiTest>>(&content) {
                    Ok(tests) => tests,
                    Err(e) => {
                        // Handle error parsing JSON
                        let error_result = create_test_result(
                            test_id.clone(),
                            TestType::Api,
                            TestStatus::Error,
                            None,
                            Some(format!("Error parsing test file: {}", e)),
                        );
                        state.add_test_result(error_result.clone()).await;
                        
                        let error_update = create_test_update(
                            test_id.clone(),
                            TestType::Api,
                            TestStatus::Error,
                            0.0,
                            None,
                            Some(format!("Error parsing test file: {}", e)),
                        );
                        let _ = state.send_test_update(error_update).await;
                        
                        // Mark test as not running
                        state.is_running.store(false, Ordering::SeqCst);
                        return;
                    }
                },
                Err(e) => {
                    // Handle error reading file
                    let error_result = create_test_result(
                        test_id.clone(),
                        TestType::Api,
                        TestStatus::Error,
                        None,
                        Some(format!("Error reading test file: {}", e)),
                    );
                    state.add_test_result(error_result.clone()).await;
                    
                    let error_update = create_test_update(
                        test_id.clone(),
                        TestType::Api,
                        TestStatus::Error,
                        0.0,
                        None,
                        Some(format!("Error reading test file: {}", e)),
                    );
                    let _ = state.send_test_update(error_update).await;
                    
                    // Mark test as not running
                    state.is_running.store(false, Ordering::SeqCst);
                    return;
                }
            };
            
            let total_tests = api_tests.len();
            let mut durations = Vec::new();
            let mut status_codes = std::collections::HashMap::new();
            let mut _successful_tests = 0;
            let mut failed_tests = 0;
            
            let client = reqwest::Client::new();
            
            // Run API tests
            for (i, test) in api_tests.iter().enumerate() {
                let progress = (i as f32) / (total_tests as f32) * 100.0;
                
                // Run the test
                match send_api_request(&client, test).await {
                    Ok(result) => {
                        durations.push(result.duration);
                        *status_codes.entry(result.status).or_insert(0) += 1;
                        
                        // Check if test passed
                        let passed = result.status == test.expected_status;
                        
                        if passed {
                            _successful_tests += 1;
                        } else {
                            failed_tests += 1;
                        }
                        
                        // Create metrics
                        let metrics = crate::model::test::create_test_metrics(
                            (i + 1) as u32,
                            total_tests as u32,
                            &durations,
                            status_codes.clone(),
                            failed_tests,
                        );
                        
                        // Send update
                        let update = create_test_update(
                            test_id.clone(),
                            TestType::Api,
                            TestStatus::Running,
                            progress,
                            Some(metrics),
                            None,
                        );
                        let _ = state.send_test_update(update).await;
                    },
                    Err(e) => {
                        failed_tests += 1;
                        
                        // Send update with error
                        let update = create_test_update(
                            test_id.clone(),
                            TestType::Api,
                            TestStatus::Running,
                            progress,
                            None,
                            Some(format!("Error in test '{}': {}", test.name, e)),
                        );
                        let _ = state.send_test_update(update).await;
                    }
                }
            }
            
            // Create final metrics
            let final_metrics = crate::model::test::create_test_metrics(
                total_tests as u32,
                total_tests as u32,
                &durations,
                status_codes,
                failed_tests,
            );
            
            // Determine final status
            let final_status = if failed_tests == 0 {
                TestStatus::Completed
            } else {
                TestStatus::Error
            };
            
            // Create final result
            let final_result = create_test_result(
                test_id.clone(),
                TestType::Api,
                final_status,
                Some(final_metrics.clone()),
                None,
            );
            
            // Add final result
            state.add_test_result(final_result.clone()).await;
            
            // Send final update
            let final_update = create_test_update(
                test_id,
                TestType::Api,
                final_status,
                100.0,
                Some(final_metrics),
                None,
            );
            let _ = state.send_test_update(final_update).await;
            
            // Mark test as not running
            state.is_running.store(false, Ordering::SeqCst);
        }
    });
    
    // Return the initial result
    Json(create_api_response(
        true,
        "API test started".to_string(),
        Some(initial_result.id),
    ))
} 