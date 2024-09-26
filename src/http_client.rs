use reqwest::Client;
use tokio::time::Instant;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::metrics::Metrics;
use serde_json::Value;
use rand::seq::SliceRandom;
use colored::*;
use std::sync::atomic::{AtomicBool, Ordering};

fn status_color(status: u16) -> ColoredString {
    match status {
        200..=299 => status.to_string().green(),
        300..=399 => status.to_string().yellow(),
        400..=599 => status.to_string().red(),
        _ => status.to_string().normal(),
    }
}

pub async fn send_request(client: &Client, url: &str, metrics: &Arc<Mutex<Metrics>>) {
    let start = Instant::now();
    match client.get(url).send().await {
        Ok(response) => {
            let duration = start.elapsed();
            let status = response.status().as_u16();
            let json = response.json::<Value>().await.ok();
            
            let mut metrics = metrics.lock().await;
            metrics.update(duration, status, json);
            
            // Print progress for each request with colored status
            println!("Request to: {} - Status: {}, Duration: {:?}", url, status_color(status), duration);
        }
        Err(_) => {
            let mut metrics = metrics.lock().await;
            metrics.update(start.elapsed(), 0, None);
            
            // Print progress for failed request
            println!("{}", "Request failed".red());
        }
    }
}

pub async fn load_test(
    urls: Arc<Vec<String>>,
    total_requests: u32,
    concurrency: u32,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
) {
    let client = Client::new();

    let mut tasks = Vec::new();
    for _ in 0..total_requests {
        let client = client.clone();
        let urls = Arc::clone(&urls);
        let metrics = Arc::clone(&metrics);

        let task = tokio::task::spawn(async move {
            let url = urls.as_slice().choose(&mut rand::thread_rng()).unwrap();
            send_request(&client, url, &metrics).await;
        });
        tasks.push(task);

        if tasks.len() as u32 == concurrency {
            for task in tasks.drain(..) {
                task.await.unwrap();
            }
        }

        // Add a small delay to prevent overwhelming the system
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // Wait for any remaining tasks
    for task in tasks {
        task.await.unwrap();
    }

    is_finished.store(true, Ordering::SeqCst);
}

pub async fn stress_test(
    urls: Arc<Vec<String>>,
    concurrency: u32,
    duration: u64,
    metrics: Arc<Mutex<Metrics>>,
    is_finished: Arc<AtomicBool>,
) {
    let client = Client::new();
    let end_time = Instant::now() + std::time::Duration::from_secs(duration);

    let mut tasks = Vec::new();

    while Instant::now() < end_time {
        let client = client.clone();
        let urls = Arc::clone(&urls);
        let metrics = Arc::clone(&metrics);

        let task = tokio::task::spawn(async move {
            let url = urls.as_slice().choose(&mut rand::thread_rng()).unwrap();
            send_request(&client, url, &metrics).await;
        });
        tasks.push(task);

        if tasks.len() as u32 == concurrency {
            for task in tasks.drain(..) {
                task.await.unwrap();
            }
        }

        // Add a small delay to prevent overwhelming the system
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    // Wait for any remaining tasks
    for task in tasks {
        task.await.unwrap();
    }

    is_finished.store(true, Ordering::SeqCst);
}