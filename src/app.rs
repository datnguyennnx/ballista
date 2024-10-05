use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::prelude::*;
use crate::core::{
    test_runner::{run_load_test, run_stress_test},
    api_test_runner::run_api_tests,
};
use crate::monitoring::resource::monitor_resources;
use crate::output::printer::format_test_results;

// Pure function to create a resource monitoring future
fn create_resource_monitor(duration: Duration) -> impl Future<Output = (Vec<f64>, Vec<f64>, Vec<(f64, f64)>)> {
    let is_finished = Arc::new(AtomicBool::new(false));
    let monitor_future = monitor_resources(Arc::clone(&is_finished));
    
    async move {
        tokio::select! {
            _ = tokio::time::sleep(duration) => {
                is_finished.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            result = monitor_future => result,
        }
    }
}

// Higher-order function to run a test with resource monitoring
async fn run_test_with_monitoring<F, Fut, T>(
    test_fn: F,
    duration: Option<Duration>,
) -> Result<String, AppError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
    T: AsRef<str>,
{
    let monitoring_duration = duration.unwrap_or(Duration::from_secs(60));
    let (test_result, resource_samples) = tokio::join!(
        test_fn(),
        create_resource_monitor(monitoring_duration)
    );

    test_result.and_then(|test_output| {
        let (cpu_samples, memory_samples, network_samples) = resource_samples;
        let resource_output = format_test_results(None, Some(monitoring_duration), &cpu_samples, &memory_samples, &network_samples);
        Ok(format!("{}\n{}", test_output.as_ref(), resource_output))
    })
}

// Pure function to get the test function based on the command
fn get_test_function(command: &Command) -> Box<dyn FnOnce() -> Box<dyn Future<Output = Result<String, AppError>> + Send + 'static> + Send + 'static> {
    match command {
        Command::LoadTest { url, requests, concurrency } => {
            let url = url.clone();
            let requests = *requests;
            let concurrency = *concurrency;
            Box::new(move || Box::new(async move {
                run_load_test(&url, requests, concurrency).await.map(|_| "Load test completed".to_string())
            }))
        },
        Command::StressTest { sitemap, duration, concurrency } => {
            let sitemap = sitemap.clone();
            let duration = *duration;
            let concurrency = *concurrency;
            Box::new(move || Box::new(async move {
                run_stress_test(&sitemap, duration, concurrency).await.map(|_| "Stress test completed".to_string())
            }))
        },
        Command::ApiTest { path } => {
            let path = path.clone();
            Box::new(move || Box::new(run_api_tests(&path)))
        },
        Command::ResourceUsage => {
            Box::new(|| Box::new(async { Ok("Resource usage monitoring completed".to_string()) }))
        },
    }
}

// Pure function to get the duration based on the command
fn get_duration(command: &Command) -> Option<Duration> {
    match command {
        Command::StressTest { duration, .. } => Some(Duration::from_secs(*duration)),
        Command::ResourceUsage => Some(Duration::from_secs(60)),
        _ => None,
    }
}

// Function composition for running the application
fn compose_application(args: Args) -> impl Future<Output = Result<String, AppError>> {
    let test_fn = get_test_function(&args.command);
    let duration = get_duration(&args.command);
    run_test_with_monitoring(test_fn, duration)
}

// Main application function using function composition
pub async fn run_application(args: Args) -> Result<String, AppError> {
    compose_application(args).await
}