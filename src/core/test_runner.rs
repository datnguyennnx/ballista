use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::core::error::AppError;
use crate::http::{TestConfig, stress_test, load_test};
use crate::metrics::Metrics;
use crate::monitoring::ResourceMonitor;
use crate::output::printer::print_test_results;
use crate::utils::formatters::format_duration;

async fn run_test(config: TestConfig, metrics: Arc<Mutex<Metrics>>, is_finished: Arc<AtomicBool>) {
    match (config.duration, config.total_requests) {
        (Some(duration), _) => stress_test(config.urls, config.concurrency, duration, metrics, is_finished).await,
        (None, Some(total_requests)) => load_test(config.urls, total_requests, config.concurrency, metrics, is_finished).await,
        _ => panic!("Invalid test configuration: either duration or total_requests must be set"),
    }
}

fn create_load_test_config(url: String, requests: u32, concurrency: u32) -> TestConfig {
    TestConfig {
        urls: Arc::new(vec![url]),
        concurrency,
        total_requests: Some(requests),
        duration: None,
    }
}

fn create_stress_test_config(sitemap: String, duration: u64, concurrency: u32) -> TestConfig {
    TestConfig {
        urls: Arc::new(vec![sitemap]), // We'll need to update this to parse the sitemap
        concurrency,
        total_requests: None,
        duration: Some(duration),
    }
}

fn print_load_test_info(url: &str, requests: u32, concurrency: u32) {
    println!("\nLoad Test");
    println!("URL to test: {}", url);
    println!("Total requests: {}", requests);
    println!("Concurrency: {}", concurrency);
    println!();
}

fn print_stress_test_info(sitemap: &str, duration: u64, concurrency: u32) {
    println!("\nStress Test");
    println!("Sitemap: {}", sitemap);
    println!("Duration: {}", format_duration(std::time::Duration::from_secs(duration)));
    println!("Concurrency: {}", concurrency);
    println!();
}

pub async fn run_load_test(url: &str, requests: u32, concurrency: u32) -> Result<(), AppError> {
    print_load_test_info(url, requests, concurrency);

    let metrics = Arc::new(Mutex::new(Metrics::new()));
    let is_finished = Arc::new(AtomicBool::new(false));

    let start = Instant::now();
    let config = create_load_test_config(url.to_string(), requests, concurrency);

    let (_, (cpu_samples, memory_samples, network_samples)) = tokio::join!(
        run_test(config, Arc::clone(&metrics), Arc::clone(&is_finished)),
        ResourceMonitor::new(Arc::clone(&is_finished)).start()
    );

    let total_duration = start.elapsed();
    let metrics = metrics.lock().await;
    let summary = metrics.summary();

    print_test_results(Some(&summary), Some(total_duration), &cpu_samples, &memory_samples, &network_samples);

    Ok(())
}

pub async fn run_stress_test(sitemap: &str, duration: u64, concurrency: u32) -> Result<(), AppError> {
    print_stress_test_info(sitemap, duration, concurrency);

    let metrics = Arc::new(Mutex::new(Metrics::new()));
    let is_finished = Arc::new(AtomicBool::new(false));

    let start = Instant::now();
    let config = create_stress_test_config(sitemap.to_string(), duration, concurrency);

    let (_, (cpu_samples, memory_samples, network_samples)) = tokio::join!(
        run_test(config, Arc::clone(&metrics), Arc::clone(&is_finished)),
        ResourceMonitor::new(Arc::clone(&is_finished)).start()
    );

    let total_duration = start.elapsed();
    let metrics = metrics.lock().await;
    let summary = metrics.summary();

    print_test_results(Some(&summary), Some(total_duration), &cpu_samples, &memory_samples, &network_samples);

    Ok(())
}