use reqwest::Client;
use tokio::time::Instant;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::metrics::Metrics;
use serde_json::Value;
use rand::seq::SliceRandom;
use colored::*;
use std::sync::atomic::{AtomicBool, Ordering};
use futures::stream::{self, StreamExt};

fn status_color(status: u16) -> ColoredString {
    match status {
        200..=299 => status.to_string().green(),
        300..=399 => status.to_string().yellow(),
        400..=599 => status.to_string().red(),
        _ => status.to_string().normal(),
    }
}

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

    match status {
        0 => println!("{}", "Request failed".red()),
        _ => println!("Request to: {} - Status: {}, Duration: {:?}", url, status_color(status), duration),
    }
}

async fn perform_test<F>(
    urls: Arc<Vec<String>>,
    concurrency: u32,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
    should_continue: F,
) where
    F: Fn() -> bool + Send + Sync + 'static,
{
    let client = Arc::new(Client::new());

    stream::iter(std::iter::repeat_with(|| ()))
        .take_while(move |_| futures::future::ready(should_continue()))
        .for_each_concurrent(concurrency as usize, |_| {
            let client = Arc::clone(&client);
            let urls = Arc::clone(&urls);
            let metrics = Arc::clone(&metrics);
            async move {
                let url = urls.choose(&mut rand::thread_rng()).unwrap();
                send_request(&client, url, &metrics).await;
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await;

    is_finished.store(true, Ordering::SeqCst);
}

pub async fn load_test(
    urls: Arc<Vec<String>>,
    total_requests: u32,
    concurrency: u32,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
) {
    let request_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let should_continue = move || {
        request_count.fetch_add(1, Ordering::SeqCst) < total_requests
    };

    perform_test(urls, concurrency, metrics, is_finished, should_continue).await;
}

pub async fn stress_test(
    urls: Arc<Vec<String>>,
    concurrency: u32,
    duration: u64,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
) {
    let end_time = Instant::now() + std::time::Duration::from_secs(duration);
    let should_continue = move || Instant::now() < end_time;

    perform_test(urls, concurrency, metrics, is_finished, should_continue).await;
}