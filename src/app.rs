use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::prelude::*;
use crate::core::{
    test_runner::{run_load_test, run_stress_test},
    api_test_runner::run_api_tests,
};
use crate::output::printer::format_test_results;


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
        
    }
}

// Pure function to get the duration based on the command
fn get_duration(command: &Command) -> Option<Duration> {
    match command {
        Command::StressTest { duration, .. } => Some(Duration::from_secs(*duration)),
        _ => None,
    }
}

// Function composition for running the application
fn compose_application(args: Args) -> impl Future<Output = Result<String, AppError>> {
    let test_fn = get_test_function(&args.command);
    let duration = get_duration(&args.command);
}

// Main application function using function composition
pub async fn run_application(args: Args) -> Result<String, AppError> {
    compose_application(args).await
}