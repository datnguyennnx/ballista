use clap::Parser;
use std::sync::Arc;
use tokio::time::Instant;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
use std::path::Path;
use std::time::Duration;
use std::fs;

mod api;
mod args;
mod http;
mod metrics;
mod monitoring;
mod output;
mod utils;

use args::Args;
use metrics::Metrics;
use http::{load_test, stress_test, TestConfig};
use utils::parsers::{parse_sitemap, UtilError};
use utils::formatters::format_duration;
use output::printer::print_test_results;
use monitoring::ResourceMonitor;
use api::{ReqwestClient, run_tests, load_tests_from_json, analyze_results};

#[derive(Debug)]
enum AppError {
    ArgValidation(String),
    NoUrls,
    Util(UtilError),
    Other(Box<dyn std::error::Error>),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::ArgValidation(msg) => write!(f, "Argument validation error: {}", msg),
            AppError::NoUrls => write!(f, "No valid URLs found to test"),
            AppError::Util(e) => write!(f, "Utility error: {}", e),
            AppError::Other(e) => write!(f, "Error: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<UtilError> for AppError {
    fn from(error: UtilError) -> Self {
        AppError::Util(error)
    }
}

async fn parse_arguments() -> Result<Args, AppError> {
    let args = Args::parse();
    if let Some(config_path) = args.config() {
        Args::from_json(Path::new(config_path)).map_err(|e| AppError::ArgValidation(e))
    } else {
        args.validate().map_err(AppError::ArgValidation)?;
        Ok(args)
    }
}

fn prepare_urls(args: &Args) -> Result<Arc<Vec<String>>, AppError> {
    let urls = if let Some(sitemap_path) = args.sitemap() {
        parse_sitemap(sitemap_path)?
    } else if let Some(url) = args.url() {
        vec![url.clone()]
    } else {
        return Err(AppError::ArgValidation("Either a sitemap or a URL must be provided".into()));
    };

    if urls.is_empty() {
        Err(AppError::NoUrls)
    } else {
        Ok(Arc::new(urls))
    }
}

async fn run_test(config: TestConfig, metrics: Arc<Mutex<Metrics>>, is_finished: Arc<AtomicBool>) {
    if let Some(duration) = config.duration {
        stress_test(config.urls, config.concurrency, duration, metrics, is_finished).await;
    } else {
        load_test(config.urls, config.total_requests, config.concurrency, metrics, is_finished).await;
    }
}

async fn run_api_tests(api_test_path: &str) -> Result<(), AppError> {
    let json_content = fs::read_to_string(api_test_path)
        .map_err(|e| AppError::Other(Box::new(e)))?;
    
    let tests = load_tests_from_json(&json_content)
        .map_err(|e| AppError::Other(Box::new(e)))?;

    let client = ReqwestClient::new();
    let results = run_tests(&client, &tests).await;

    let (total, successful, total_duration) = analyze_results(&results);

    println!("\nAPI Test Results");
    println!("Total tests: {}", total);
    println!("Successful tests: {}", successful);
    println!("Total duration: {}", format_duration(total_duration));

    for result in results {
        println!("\nTest: {}", result.name);
        println!("Success: {}", result.success);
        println!("Duration: {}", format_duration(result.duration));
        println!("Status: {}", result.status);
        if let Some(error) = result.error {
            println!("Error: {}", error);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let args = parse_arguments().await?;
    let is_finished = Arc::new(AtomicBool::new(false));

    if let Some(api_test_path) = args.api_test() {
        run_api_tests(api_test_path).await?;
    } else if args.resource_usage() {
        println!("Collecting resource usage data for 60 seconds");
        let resource_monitor = ResourceMonitor::new(Arc::clone(&is_finished));
        let resource_monitor_handle = tokio::spawn(resource_monitor.start());
        tokio::time::sleep(Duration::from_secs(60)).await;
        is_finished.store(true, std::sync::atomic::Ordering::SeqCst);
        let (cpu_samples, memory_samples, network_samples) = resource_monitor_handle.await.map_err(|e| AppError::Other(Box::new(e)))?;
        print_test_results(None, None, &cpu_samples, &memory_samples, &network_samples);
    } else {
        let urls = prepare_urls(&args)?;
        let metrics = Arc::new(Mutex::new(Metrics::new()));

        println!("\n{}", if args.stress() { "Stress Test" } else { "Load Test" });
        println!("URLs to test: {}", urls.len());
        println!("Concurrency: {}", args.concurrency());
        if args.stress() {
            println!("Duration: {}", format_duration(std::time::Duration::from_secs(args.duration())));
        } else {
            println!("Total requests: {}", args.requests());
        }
        println!();

        let start = Instant::now();

        let config = TestConfig {
            urls: Arc::clone(&urls),
            concurrency: args.concurrency(),
            total_requests: args.requests(),
            duration: if args.stress() { Some(args.duration()) } else { None },
        };

        let test_handle = tokio::spawn(run_test(config, Arc::clone(&metrics), Arc::clone(&is_finished)));

        let resource_monitor = ResourceMonitor::new(Arc::clone(&is_finished));
        let resource_monitor_handle = tokio::spawn(resource_monitor.start());

        test_handle.await.map_err(|e| AppError::Other(Box::new(e)))?;
        let (cpu_samples, memory_samples, network_samples) = resource_monitor_handle.await.map_err(|e| AppError::Other(Box::new(e)))?;
        let total_duration = start.elapsed();

        let metrics = metrics.lock().await;
        let summary = metrics.summary();

        print_test_results(Some(&summary), Some(total_duration), &cpu_samples, &memory_samples, &network_samples);
    }

    Ok(())
}
