use reqwest::Client;
use tokio::time::Instant;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::metrics::Metrics;
use serde_json::Value;
use rand::seq::SliceRandom;
use std::sync::atomic::{AtomicBool, Ordering};
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};

async fn send_request(client: &Client, url: &str, metrics: &Arc<Mutex<Metrics>>) {
    let start = Instant::now();
    let result = client.get(url).send().await;

    let (duration, status, json) = match result {
        Ok(response) => {
            let status = response.status().as_u16();
            let json = response.json::<Value>().await.ok();
            (start.elapsed(), status, json)
        }
        Err(_) => (start.elapsed(), 0, None),
    };

    metrics.lock().await.update(duration, status, json);
}

async fn perform_test<F>(
    urls: Arc<Vec<String>>,
    concurrency: u32,
    total_requests: u32,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
    should_continue: F,
) where
    F: Fn() -> bool + Send + Sync + 'static,
{
    let client = Arc::new(Client::new());
    let pb = ProgressBar::new(total_requests as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .expect("Failed to set progress bar template")
        .progress_chars("##-"));

    let request_count = Arc::new(std::sync::atomic::AtomicU32::new(0));

    stream::iter(std::iter::repeat_with(|| ()))
        .take_while(move |_| futures::future::ready(should_continue()))
        .for_each_concurrent(concurrency as usize, |_| {
            let client = Arc::clone(&client);
            let urls = Arc::clone(&urls);
            let metrics = Arc::clone(&metrics);
            let pb = pb.clone();
            let request_count = Arc::clone(&request_count);
            async move {
                let url = urls.choose(&mut rand::thread_rng()).unwrap();
                send_request(&client, url, &metrics).await;
                let count = request_count.fetch_add(1, Ordering::SeqCst) + 1;
                pb.set_position(count as u64);
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
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
    let request_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let should_continue = move || {
        request_count.fetch_add(1, Ordering::SeqCst) < total_requests
    };

    perform_test(urls, concurrency, total_requests, metrics, is_finished, should_continue).await;
}

pub async fn stress_test(
    urls: Arc<Vec<String>>,
    concurrency: u32,
    duration: u64,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
) {
    println!("Starting stress test for {} seconds with {} concurrent requests", duration, concurrency);
    let end_time = Instant::now() + std::time::Duration::from_secs(duration);
    let should_continue = move || Instant::now() < end_time;

    // For stress test, we'll use a large number as total_requests
    let total_requests = u32::MAX;
    perform_test(urls, concurrency, total_requests, metrics, is_finished, should_continue).await;
}