use reqwest::Client;
use tokio::time::Instant;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::metrics::Metrics;
use serde_json::Value;
use rand::seq::SliceRandom;
use std::sync::atomic::{AtomicBool, Ordering};
use futures::{stream, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug)]
pub struct RequestResult {
    pub duration: std::time::Duration,
    pub status: u16,
    pub json: Option<Value>,
}

#[derive(Clone)]
pub struct TestConfig {
    pub urls: Arc<Vec<String>>,
    pub concurrency: u32,
    pub total_requests: u32,
    pub duration: Option<u64>,
}

async fn send_request(client: &Client, url: &str) -> Result<RequestResult, reqwest::Error> {
    let start = Instant::now();
    let response = client.get(url).send().await?;
    let status = response.status().as_u16();
    let json = response.json::<Value>().await.ok();
    Ok(RequestResult {
        duration: start.elapsed(),
        status,
        json,
    })
}

fn create_progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .expect("Failed to set progress bar template")
        .progress_chars("##-"));
    pb
}

async fn perform_test(
    config: TestConfig,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
) {
    let client = Arc::new(Client::new());
    let pb = create_progress_bar(config.total_requests as u64);

    let requests = stream::iter(0..config.total_requests)
        .map(|_| {
            let client = Arc::clone(&client);
            let urls = Arc::clone(&config.urls);
            async move {
                let url = urls.choose(&mut rand::thread_rng()).unwrap();
                send_request(&client, url).await
            }
        })
        .buffer_unordered(config.concurrency as usize);

    requests
        .for_each(|result| {
            let metrics = Arc::clone(&metrics);
            let pb = pb.clone();
            async move {
                if let Ok(req_result) = result {
                    let mut metrics = metrics.lock().await;
                    metrics.add_request(req_result.duration, req_result.status, req_result.json);
                }
                pb.inc(1);
            }
        })
        .await;

    pb.finish_with_message("Test completed");
    is_finished.store(true, Ordering::SeqCst);
}

pub async fn load_test(
    urls: Arc<Vec<String>>,
    total_requests: u32,
    concurrency: u32,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
) {
    println!("Starting load test with {} total requests and {} concurrent requests", total_requests, concurrency);
    let config = TestConfig {
        urls,
        concurrency,
        total_requests,
        duration: None,
    };
    perform_test(config, metrics, is_finished).await;
}

pub async fn stress_test(
    urls: Arc<Vec<String>>,
    concurrency: u32,
    duration: u64,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
) {
    println!("Starting stress test for {} seconds with {} concurrent requests", duration, concurrency);
    let config = TestConfig {
        urls,
        concurrency,
        total_requests: u32::MAX,
        duration: Some(duration),
    };
    let end_time = Instant::now() + std::time::Duration::from_secs(duration);
    
    tokio::select! {
        _ = perform_test(config, metrics, Arc::clone(&is_finished)) => {},
        _ = tokio::time::sleep_until(end_time) => {
            println!("Stress test duration reached");
            is_finished.store(true, Ordering::SeqCst);
        }
    }
}