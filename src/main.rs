use clap::Parser;
use std::sync::Arc;
use tokio::time::Instant;
use std::sync::atomic::AtomicBool;
use colored::*;

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
    
    let urls = match (args.sitemap.as_ref(), args.url.as_ref()) {
        (Some(sitemap), _) => {
            println!("Parsing sitemap: {}", sitemap);
            parse_sitemap(sitemap).map_err(|e| format!("Failed to parse sitemap: {}", e))?
        },
        (_, Some(url)) => vec![url.clone()],
        _ => return Err("Either --url or --sitemap must be provided".into()),
    };

    if urls.is_empty() {
        return Err("No valid URLs found to test".into());
    }

    let urls = Arc::new(urls);
    let metrics = Metrics::new();
    let is_finished = Arc::new(AtomicBool::new(false));

    println!("\nStarting {}...", if args.stress { "stress test" } else { "load test" });
    let start = Instant::now();
    
    let test_handle = tokio::spawn({
        let urls = Arc::clone(&urls);
        let metrics = Arc::clone(&metrics);
        let is_finished = Arc::clone(&is_finished);
        async move {
            if args.stress {
                stress_test(urls, args.concurrency, args.duration, metrics, is_finished).await;
            } else {
                load_test(urls, args.requests, args.concurrency, metrics, is_finished).await;
            }
        }
    });

    let resource_monitor_handle = tokio::spawn({
        let is_finished = Arc::clone(&is_finished);
        async move {
            let mut cpu_samples = Vec::new();
            let mut memory_samples = Vec::new();
            while !is_finished.load(std::sync::atomic::Ordering::Relaxed) {
                if let Ok(cpu) = get_cpu_usage() {
                    cpu_samples.push(cpu);
                }
                if let Ok(memory) = get_memory_usage() {
                    memory_samples.push(memory);
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            (cpu_samples, memory_samples)
        }
    });

    test_handle.await?;
    let (cpu_samples, memory_samples) = resource_monitor_handle.await?;
    let total_duration = start.elapsed();

    println!("\nTest completed. Analyzing results...");

    let metrics = metrics.lock().await;
    let summary = metrics.summary();

    println!("\nResults:");
    println!(" - Total requests: {}", summary.total_requests);
    println!(" - Successful requests: {}", summary.successful_requests);
    println!(" - Failed requests: {}", summary.failed_requests);
    println!(" - Total time: {:.2?}", total_duration);
    println!(" - Requests per second: {:.2}", summary.total_requests as f64 / total_duration.as_secs_f64());
    println!(" - Average response time: {:.2?}", summary.total_time / summary.total_requests);
    
    if let Some(min_duration) = summary.min_duration {
        println!(" - Minimum response time: {:.2?}", min_duration);
    }
    if let Some(max_duration) = summary.max_duration {
        println!(" - Maximum response time: {:.2?}", max_duration);
    }
    if let Some(median_duration) = summary.median_duration {
        println!(" - Median response time: {:.2?}", median_duration);
    }
    if let Some(percentile_95) = summary.percentile_95 {
        println!(" - 95th percentile response time: {:.2?}", percentile_95);
    }

    println!("\nHTTP Status Codes:");
    for (status, count) in &summary.status_codes {
        println!("  Status {}: {}", status_color(*status), count);
    }

    println!("\nResource Usage:");
    if !cpu_samples.is_empty() {
        println!("Average CPU Usage: {:.2}%", cpu_samples.iter().sum::<f64>() / cpu_samples.len() as f64);
    } else {
        println!("CPU Usage: Unable to collect data");
    }
    if !memory_samples.is_empty() {
        println!("Average Memory Usage: {:.2}%", memory_samples.iter().sum::<f64>() / memory_samples.len() as f64);
    } else {
        println!("Memory Usage: Unable to collect data");
    }

    Ok(())
}

fn status_color(status: u16) -> ColoredString {
    match status {
        200..=299 => status.to_string().green(),
        300..=399 => status.to_string().yellow(),
        400..=599 => status.to_string().red(),
        _ => status.to_string().normal(),
    }
}
