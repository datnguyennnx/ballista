use reqwest::{Client, Method};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use rand::seq::SliceRandom;
use std::time::Instant;
use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};

use crate::model::error::AppError;
use crate::model::test::{ApiTest, RequestResult, TestConfig, TestMetrics};

/// Convert a string to reqwest Method
pub fn string_to_method(method: &str) -> Result<Method, AppError> {
    match method.to_uppercase().as_str() {
        "GET" => Ok(Method::GET),
        "POST" => Ok(Method::POST),
        "PUT" => Ok(Method::PUT),
        "DELETE" => Ok(Method::DELETE),
        "PATCH" => Ok(Method::PATCH),
        "HEAD" => Ok(Method::HEAD),
        "OPTIONS" => Ok(Method::OPTIONS),
        _ => Err(AppError::InvalidConfig(format!("Invalid HTTP method: {}", method))),
    }
}

/// Send a simple HTTP request and return the result
pub async fn send_request(client: &Client, url: &str) -> Result<RequestResult, AppError> {
    let start = Instant::now();
    
    match client.get(url).send().await {
        Ok(response) => {
            let status = response.status();
            let duration = start.elapsed();
            
            let json = if status.is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => Some(json),
                    Err(_) => None,
                }
            } else {
                None
            };
            
            Ok(RequestResult {
                duration,
                status: status.as_u16(),
                json,
                error: None,
            })
        },
        Err(e) => Err(AppError::NetworkError(e)),
    }
}

/// Send an API request based on an ApiTest specification
pub async fn send_api_request(client: &Client, api_test: &ApiTest) -> Result<RequestResult, AppError> {
    let start = Instant::now();
    let method = string_to_method(&api_test.method)?;

    let mut request = client.request(method, &api_test.url);
    
    // Add headers if specified
    if let Some(headers) = &api_test.headers {
        for (key, value) in headers {
            request = request.header(key, value);
        }
    }

    // Add body if specified
    if let Some(body) = &api_test.body {
        request = request.json(body);
    }

    // Send the request
    match request.send().await {
        Ok(response) => {
            let status = response.status();
            let duration = start.elapsed();
            
            // Try to parse JSON response if successful
            let json = if status.is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => Some(json),
                    Err(_) => None,
                }
            } else {
                None
            };
            
            Ok(RequestResult {
                duration,
                status: status.as_u16(),
                json,
                error: None,
            })
        },
        Err(e) => Err(AppError::NetworkError(e)),
    }
}

/// Run a load test with the specified configuration
pub async fn load_test(
    client: &Client,
    config: &TestConfig,
    metrics: Arc<Mutex<TestMetrics>>,
    is_finished: Arc<AtomicBool>,
) -> Result<(), AppError> {
    if config.urls.is_empty() {
        return Err(AppError::NoUrls);
    }
    
    let total_requests = config.total_requests.ok_or_else(|| 
        AppError::ConfigError("Total requests must be specified for load test".to_string()))?;
    
    println!("Starting load test with {} total requests and {} concurrent requests", 
        total_requests, config.concurrency);
    
    perform_test(client, config, metrics, is_finished).await
}

/// Run a stress test with the specified configuration
pub async fn stress_test(
    client: &Client,
    config: &TestConfig,
    metrics: Arc<Mutex<TestMetrics>>,
    is_finished: Arc<AtomicBool>,
) -> Result<(), AppError> {
    if config.urls.is_empty() {
        return Err(AppError::NoUrls);
    }
    
    let duration = config.duration.ok_or_else(|| 
        AppError::ConfigError("Duration must be specified for stress test".to_string()))?;
    
    println!("Starting stress test for {} seconds with {} concurrent requests", 
        duration, config.concurrency);
    
    let end_time = tokio::time::Instant::now() + std::time::Duration::from_secs(duration);
    
    tokio::select! {
        result = perform_test(client, config, metrics, Arc::clone(&is_finished)) => result,
        _ = tokio::time::sleep_until(end_time) => {
            println!("Stress test duration reached");
            is_finished.store(true, Ordering::SeqCst);
            Ok(())
        }
    }
}

/// Perform a test with the specified configuration
async fn perform_test(
    client: &Client,
    config: &TestConfig,
    metrics: Arc<Mutex<TestMetrics>>,
    is_finished: Arc<AtomicBool>,
) -> Result<(), AppError> {
    let pb = config.total_requests.map(|total| create_progress_bar(total as u64));
    
    let requests = stream::iter(std::iter::repeat(()).take(config.total_requests.unwrap_or(u32::MAX) as usize))
        .map(|_| {
            let urls = &config.urls;
            let client = client.clone();
            async move {
                let url = urls.choose(&mut rand::thread_rng()).unwrap();
                send_request(&client, url).await
            }
        })
        .buffer_unordered(config.concurrency as usize)
        .take_while(|_| async {
            !is_finished.load(Ordering::SeqCst)
        });

    let pb_clone = pb.clone();
    requests
        .fold(metrics, |metrics, result| {
            let pb = pb_clone.clone();
            async move {
                if let Ok(ref req_result) = result {
                    let mut metrics_guard = metrics.lock().await;
                    add_request_metric(&mut metrics_guard, req_result);
                    
                    if let Some(ref pb) = pb {
                        pb.inc(1);
                    }
                }
                metrics
            }
        })
        .await;

    if let Some(pb) = pb {
        pb.finish_with_message("Load test completed");
    }

    Ok(())
}

// Pure function to update metrics with a request result
fn add_request_metric(metrics: &mut TestMetrics, result: &RequestResult) {
    metrics.requests_completed += 1;
    
    // Update status code counts
    let status_count = metrics.status_codes.entry(result.status).or_insert(0);
    *status_count += 1;
    
    // Convert duration to milliseconds
    let duration_ms = result.duration.as_secs_f64() * 1000.0;
    
    // Update min/max response times
    if let Some(min) = metrics.min_response_time.as_mut() {
        *min = min.min(duration_ms);
    } else {
        metrics.min_response_time = Some(duration_ms);
    }
    
    if let Some(max) = metrics.max_response_time.as_mut() {
        *max = max.max(duration_ms);
    } else {
        metrics.max_response_time = Some(duration_ms);
    }
    
    // Update average
    let count = metrics.requests_completed as f64;
    if count > 1.0 {
        let old_avg = metrics.avg_response_time;
        metrics.avg_response_time = old_avg + (duration_ms - old_avg) / count;
    } else {
        metrics.avg_response_time = duration_ms;
    }
    
    // Update error count if needed
    if result.error.is_some() {
        metrics.errors += 1;
    }
}

// Pure function to create a progress bar
fn create_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .expect("Failed to set progress bar template")
        .progress_chars("##-"));
    pb
}

// Pure function to run a test
#[allow(dead_code)]
async fn _run_test(config: &TestConfig, metrics: Arc<Mutex<TestMetrics>>, is_finished: Arc<AtomicBool>) -> Result<(), AppError> {
    let client = _create_client();
    match (config.duration, config.total_requests) {
        (Some(_duration), _) => stress_test(&client, config, metrics, is_finished).await,
        (None, Some(_total_requests)) => load_test(&client, config, metrics, is_finished).await,
        _ => Err(AppError::InvalidConfig("Invalid test configuration: either duration or total_requests must be set".to_string())),
    }
}

// Pure function to create a client
#[allow(dead_code)]
fn _create_client() -> reqwest::Client {
    reqwest::Client::new()
}