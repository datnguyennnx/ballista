use clap::Parser;
use std::sync::Arc;
use tokio::time::Instant;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;

mod args;
mod metrics;
mod http_client;
mod utils;
mod structure_output;

use args::Args;
use metrics::Metrics;
use http_client::{load_test, stress_test};
use utils::{get_cpu_usage, get_memory_usage, get_network_usage, parse_sitemap};
use structure_output::print_test_results;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Validate arguments
    if let Err(err) = args.validate() {
        return Err(err.into());
    }

    let urls = if let Some(sitemap_path) = &args.sitemap {
        parse_sitemap(sitemap_path)?
    } else if let Some(url) = &args.url {
        vec![url.clone()]
    } else {
        return Err("Either a sitemap or a URL must be provided".into());
    };

    if urls.is_empty() {
        return Err("No valid URLs found to test".into());
    }

    let urls = Arc::new(urls);
    let metrics = Arc::new(Mutex::new(Metrics::default()));
    let is_finished = Arc::new(AtomicBool::new(false));

    println!("\n{}", if args.stress { "Stress Test" } else { "Load Test" });
    println!("URLs to test: {}", urls.len());
    println!("Concurrency: {}", args.concurrency);
    if args.stress {
        println!("Duration: {} seconds", args.duration);
    } else {
        println!("Total requests: {}", args.requests);
    }
    println!();

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
            let mut network_samples = Vec::new();

            while !is_finished.load(std::sync::atomic::Ordering::Relaxed) {
                if let Ok(cpu) = get_cpu_usage() {
                    cpu_samples.push(cpu);
                }
                if let Ok(memory) = get_memory_usage() {
                    memory_samples.push(memory);
                }
                if let Ok(network) = get_network_usage() {
                    network_samples.push(network);
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            (cpu_samples, memory_samples, network_samples)
        }
    });

    test_handle.await?;
    let (cpu_samples, memory_samples, network_samples) = resource_monitor_handle.await?;
    let total_duration = start.elapsed();

    let metrics = metrics.lock().await;
    let summary = metrics.summary();

    print_test_results(&summary, total_duration, &cpu_samples, &memory_samples, &network_samples);

    Ok(())
}
