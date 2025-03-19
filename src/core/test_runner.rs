use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::Mutex;
use tokio::time::{Instant, Duration};
use std::future::Future;
use std::pin::Pin;

use crate::prelude::*;
use crate::http::{TestConfig, load_test, stress_test};
use crate::output::printer::format_test_results;
use crate::metrics::collector::{Metrics, new_metrics, calculate_summary};

// Pure function to create a client
fn create_client() -> reqwest::Client {
    reqwest::Client::new()
}

// Pure function to run a test
async fn run_test(config: &TestConfig, metrics: Arc<Mutex<Metrics>>, is_finished: Arc<AtomicBool>) -> Result<(), AppError> {
    let client = create_client();
    match (config.duration, config.total_requests) {
        (Some(duration), _) => stress_test(&client, &config.urls, config.concurrency, duration, metrics, is_finished).await,
        (None, Some(total_requests)) => load_test(&client, &config.urls, total_requests, config.concurrency, metrics, is_finished).await,
        _ => Err(AppError::InvalidConfig("Invalid test configuration: either duration or total_requests must be set".to_string())),
    }
}

// Pure function to create test configuration
fn create_test_config<T: Into<String>>(urls: Vec<T>, concurrency: u32, total_requests: Option<u32>, duration: Option<u64>) -> TestConfig {
    TestConfig {
        urls: Arc::new(urls.into_iter().map(Into::into).collect()),
        concurrency,
        total_requests,
        duration,
    }
}

// Pure function to format test info
fn format_test_info<T: AsRef<str>>(test_type: &str, url_or_sitemap: T, requests_or_duration: impl ToString, concurrency: u32) -> String {
    format!(
        "\n{} Test\n{}: {}\n{}: {}\nConcurrency: {}\n",
        test_type,
        if test_type == "Load" { "URL to test" } else { "Sitemap" },
        url_or_sitemap.as_ref(),
        if test_type == "Load" { "Total requests" } else { "Duration" },
        requests_or_duration.to_string(),
        concurrency
    )
}

// Higher-order function to run a generic test
async fn run_generic_test<F, G, H>(
    config: TestConfig,
    format_info: F,
    get_duration: G,
    create_metrics: H,
) -> Result<(), AppError>
where
    F: Fn() -> String,
    G: Fn() -> Option<Duration>,
    H: Fn() -> Arc<Mutex<Metrics>>,
{
    println!("{}", format_info());

    let metrics = create_metrics();
    let is_finished = Arc::new(AtomicBool::new(false));

    let start = Instant::now();

    let test_result = run_test(&config, Arc::clone(&metrics), Arc::clone(&is_finished));

    test_result.await?;

    let total_duration = get_duration().unwrap_or_else(|| start.elapsed());
    let metrics = metrics.lock().await;
    let summary = calculate_summary(&metrics);

    println!("{}", format_test_results(Some(&summary), Some(total_duration)));

    Ok(())
}

// Composition function for running a load test
pub async fn run_load_test(url: &str, requests: u32, concurrency: u32) -> Result<(), AppError> {
    let config = create_test_config(vec![url], concurrency, Some(requests), None);
    let format_info = || format_test_info("Load", url, requests, concurrency);
    let get_duration = || None;
    let create_metrics = || Arc::new(Mutex::new(new_metrics()));
    run_generic_test(config, format_info, get_duration, create_metrics).await
}

// Composition function for running a stress test
pub async fn run_stress_test(sitemap: &str, duration: u64, concurrency: u32) -> Result<(), AppError> {
    let config = create_test_config(vec![sitemap], concurrency, None, Some(duration));
    let format_info = || format_test_info("Stress", sitemap, format_duration(Duration::from_secs(duration)), concurrency);
    let get_duration = || Some(Duration::from_secs(duration));
    let create_metrics = || Arc::new(Mutex::new(new_metrics()));
    run_generic_test(config, format_info, get_duration, create_metrics).await
}

// Higher-order function to compose test runners
pub fn compose_test_runners<F, G>(
    runner1: F,
    runner2: G,
) -> impl Fn(String, u32, u32) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'static>>
where
    F: Fn(&str, u32, u32) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'static>> + Send + Sync + Clone + 'static,
    G: Fn(&str, u32, u32) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'static>> + Send + Sync + Clone + 'static,
{
    move |url_or_sitemap, requests_or_duration, concurrency| {
        let runner1 = runner1.clone();
        let runner2 = runner2.clone();
        let url_or_sitemap_clone = url_or_sitemap.clone();
        Box::pin(async move {
            runner1(&url_or_sitemap, requests_or_duration, concurrency).await?;
            runner2(&url_or_sitemap_clone, requests_or_duration, concurrency).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_load_test() {
        let result = run_load_test("http://example.com", 10, 2).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_stress_test() {
        let result = run_stress_test("http://example.com/sitemap.xml", 5, 2).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_compose_test_runners() {
        let composed_runner = compose_test_runners(
            |url, req, conc| Box::pin(run_load_test(url, req, conc)),
            |url, dur, conc| Box::pin(run_stress_test(url, dur, conc))
        );
        let result = composed_runner("http://example.com".to_string(), 10, 2).await;
        assert!(result.is_ok());
    }
}