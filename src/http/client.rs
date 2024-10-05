use reqwest::{Client, Method, header::HeaderMap};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::prelude::*;
use crate::http::request::{TestConfig, send_request};
use crate::api::types::{ApiTest, TestResult};
use crate::metrics::collector::{Metrics, add_request as add_metric};
use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::sync::atomic::{AtomicBool, Ordering};
use rand::seq::SliceRandom;
use std::time::Instant;

pub async fn send_api_request(client: &Client, api_test: &ApiTest) -> Result<TestResult, AppError> {
    let start = Instant::now();
    let method = Method::from_bytes(api_test.method.as_bytes())
        .map_err(|e| AppError::ParseError(format!("Invalid HTTP method: {}", e)))?;

    let mut request = client.request(method, &api_test.url);
    
    if let Some(headers) = &api_test.headers {
        let header_map: HeaderMap = headers.iter()
            .map(|(k, v)| (k.parse().unwrap(), v.parse().unwrap()))
            .collect();
        request = request.headers(header_map);
    }

    if let Some(body) = &api_test.body {
        request = request.json(body);
    }

    let response = request.send().await
        .map_err(|e| AppError::Other(e.to_string()))?;

    let status = response.status();
    let duration = start.elapsed();

    let success = status.as_u16() == api_test.expected_status;

    Ok(TestResult {
        name: api_test.name.clone(),
        success,
        duration,
        status: status.as_u16(),
        error: None,
    })
}

pub async fn load_test(
    client: &Client,
    urls: &Arc<Vec<String>>,
    total_requests: u32,
    concurrency: u32,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
) -> Result<(), AppError> {
    println!("Starting load test with {} total requests and {} concurrent requests", total_requests, concurrency);
    let config = TestConfig {
        urls: Arc::clone(urls),
        concurrency,
        total_requests: Some(total_requests),
        duration: None,
    };
    perform_test(client, config, metrics, is_finished).await
}

pub async fn stress_test(
    client: &Client,
    urls: &Arc<Vec<String>>,
    concurrency: u32,
    duration: u64,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
) -> Result<(), AppError> {
    println!("Starting stress test for {} seconds with {} concurrent requests", duration, concurrency);
    let config = TestConfig {
        urls: Arc::clone(urls),
        concurrency,
        total_requests: None,
        duration: Some(duration),
    };
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

async fn perform_test(
    client: &Client,
    config: TestConfig,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
) -> Result<(), AppError> {
    let pb = config.total_requests.map(|total| create_progress_bar(total as u64));

    let requests = stream::iter(std::iter::repeat(()).take(config.total_requests.unwrap_or(u32::MAX) as usize))
        .map(|_| {
            let urls = Arc::clone(&config.urls);
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
                if let Ok(req_result) = result {
                    let mut metrics_guard = metrics.lock().await;
                    *metrics_guard = add_metric((*metrics_guard).clone(), req_result.duration, req_result.status, None);
                }
                if let Some(pb) = &pb {
                    pb.inc(1);
                }
                metrics
            }
        })
        .await;

    if let Some(pb) = pb {
        pb.finish_with_message("Test completed");
    }
    is_finished.store(true, Ordering::SeqCst);
    Ok(())
}

fn create_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .expect("Failed to set progress bar template")
        .progress_chars("##-"));
    pb
}