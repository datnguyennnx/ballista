use clap::Parser;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::Instant;
use std::sync::atomic::AtomicBool;

mod args;
mod metrics;
mod http_client;
mod utils;

use args::Args;
use metrics::Metrics;
use http_client::{load_test, stress_test};
use utils::{get_cpu_usage, get_memory_usage, parse_sitemap};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    let urls = if let Some(sitemap) = args.sitemap.as_ref() {
        println!("Parsing sitemap: {}", sitemap);
        match parse_sitemap(sitemap) {
            Ok(urls) => {
                println!("Successfully parsed {} URLs from sitemap", urls.len());
                urls
            },
            Err(e) => return Err(format!("Failed to parse sitemap: {}", e).into()),
        }
    } else if let Some(url) = args.url.as_ref() {
        vec![url.clone()]
    } else {
        return Err("Either --url or --sitemap must be provided".into());
    };

    if urls.is_empty() {
        return Err("No valid URLs found to test".into());
    }

    let urls = Arc::new(urls);
    let metrics = Arc::new(Mutex::new(Metrics::new()));
    let is_finished = Arc::new(AtomicBool::new(false));

    println!("\nStarting {}...", if args.stress { "stress test" } else { "load test" });
    let start = Instant::now();
    let metrics_clone = Arc::clone(&metrics);
    let is_finished_clone = Arc::clone(&is_finished);
    let urls_clone = Arc::clone(&urls);
    let test_handle = tokio::task::spawn(async move {
        if args.stress {
            stress_test(urls_clone, args.concurrency, args.duration, metrics_clone, is_finished_clone).await;
        } else {
            load_test(urls_clone, args.requests, args.concurrency, metrics_clone, is_finished_clone).await;
        }
    });

    let is_finished_clone = Arc::clone(&is_finished);
    let resource_monitor_handle = tokio::task::spawn(async move {
        let mut cpu_samples = Vec::new();
        let mut memory_samples = Vec::new();
        while !is_finished_clone.load(std::sync::atomic::Ordering::SeqCst) {
            if let Ok(cpu) = get_cpu_usage() {
                cpu_samples.push(cpu);
            }
            if let Ok(memory) = get_memory_usage() {
                memory_samples.push(memory);
            }
            
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        (cpu_samples, memory_samples)
    });

    test_handle.await?;
    let (cpu_samples, memory_samples) = resource_monitor_handle.await?;
    let total_duration = start.elapsed();

    println!("\nTest completed. Analyzing results...");

    let metrics = metrics.lock().await;
    let mut sorted_times = metrics.response_times.clone();
    sorted_times.sort();

    println!("\nResults:");
    println!(" - Total requests: {}", metrics.total_requests);
    println!(" - Successful requests: {}", metrics.successful_requests);
    println!(" - Failed requests: {}", metrics.failed_requests);
    println!(" - Total time: {:.2?}", total_duration);
    println!(" - Requests per second: {:.2}", metrics.total_requests as f64 / total_duration.as_secs_f64());
    println!(" - Average response time: {:.2?}", metrics.total_time / metrics.total_requests);
    if let Some(min_duration) = metrics.min_duration() {
        println!(" - Minimum response time: {:.2?}", min_duration);
    }
    if let Some(max_duration) = metrics.max_duration() {
        println!(" - Maximum response time: {:.2?}", max_duration);
    }
    if let Some(median_duration) = metrics.median_duration() {
        println!(" - Median (50th percentile) response time: {:.2?}", median_duration);
    }

    println!("\nHTTP Status Codes:");
    for (status, count) in &metrics.status_codes {
        println!("  Status:{}: {}", status, count);
    }

    println!("\nResource Usage:");
    if !cpu_samples.is_empty() {
        println!("Average CPU Usage: {:.2}%", cpu_samples.iter().sum::<f64>() / cpu_samples.len() as f64);
    } else {
        println!("CPU Usage: Unable to collect data");
    }
    if !memory_samples.is_empty() {
        println!(" - Average Memory Usage: {:.2}%", memory_samples.iter().sum::<f64>() / memory_samples.len() as f64);
    } else {
        println!(" - Memory Usage: Unable to collect data");
    }

    Ok(())
}
