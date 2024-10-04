use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use target_tool::core::{
    config::{Args, Command},
    error::AppError,
    test_runner::{run_load_test, run_stress_test},
    api_test_runner::run_api_tests,
};
use target_tool::monitoring::ResourceMonitor;
use target_tool::output::printer::print_test_results;

pub async fn run_application(args: Args) -> Result<(), AppError> {
    match args.command {
        Command::LoadTest { url, requests, concurrency } => {
            run_load_test(&url, requests, concurrency).await
        },
        Command::StressTest { sitemap, duration, concurrency } => {
            run_stress_test(&sitemap, duration, concurrency).await
        },
        Command::ApiTest { path } => run_api_tests(&path).await,
        Command::ResourceUsage => collect_resource_usage().await,
    }
}

async fn collect_resource_usage() -> Result<(), AppError> {
    const COLLECTION_DURATION: Duration = Duration::from_secs(60);
    
    println!("Collecting resource usage data for {} seconds", COLLECTION_DURATION.as_secs());
    let is_finished = Arc::new(AtomicBool::new(false));
    let resource_monitor = ResourceMonitor::new(Arc::clone(&is_finished));
    
    let (monitor_result, _): ((Vec<f64>, Vec<f64>, Vec<(f64, f64)>), _) = tokio::join!(
        resource_monitor.start(),
        tokio::time::sleep(COLLECTION_DURATION)
    );

    is_finished.store(true, Ordering::SeqCst);
    
    let (cpu_samples, memory_samples, network_samples) = monitor_result;
    print_test_results(None, None, &cpu_samples, &memory_samples, &network_samples);
    
    Ok(())
}