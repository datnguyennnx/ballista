use std::fs;
use std::path::Path;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::prelude::*;
use crate::api::{run_tests, load_tests_from_json, analyze_results};
use crate::api::types::{TestResult, ApiTest};
use crate::core::error::{AppError, to_app_error};
use crate::utils::formatters::format_duration;
use reqwest::Client;

// Pure function to read file content
fn read_file_content(path: &Path) -> Result<String, AppError> {
    // Try reading from the direct path first
    fs::read_to_string(path)
        .or_else(|_| {
            // If that fails, try reading from test_examples directory
            let test_example_path = Path::new("test_examples").join(path.file_name().unwrap());
            fs::read_to_string(test_example_path)
        })
        .map_err(to_app_error(AppError::FileError))
}

// Pure function to parse JSON and load tests
fn parse_and_load_tests(json_content: &str) -> Result<Vec<ApiTest>, AppError> {
    load_tests_from_json(json_content).map_err(to_app_error(AppError::ParseError))
}

// Pure function to create HTTP client
fn create_client() -> Client {
    Client::new()
}

// Higher-order function to compose file reading and test parsing
fn compose_read_and_parse<F, G>(read_file: F, parse_tests: G) -> impl Fn(&Path) -> Result<Vec<ApiTest>, AppError>
where
    F: Fn(&Path) -> Result<String, AppError>,
    G: Fn(&str) -> Result<Vec<ApiTest>, AppError>,
{
    move |path| {
        read_file(path).and_then(|content| parse_tests(&content))
    }
}

// Higher-order function to run tests
fn run_tests_with_client(client: Arc<Client>) -> impl Fn(Vec<ApiTest>) -> Pin<Box<dyn Future<Output = Vec<TestResult>> + Send + 'static>> {
    move |tests| {
        let client = Arc::clone(&client);
        Box::pin(async move {
            run_tests(&client, &tests).await
        })
    }
}

// Pure function to format summary
fn format_summary(total: usize, successful: usize, total_duration: Duration) -> String {
    format!(
        "\nAPI Test Results\nTotal tests: {}\nSuccessful tests: {}\nTotal duration: {}",
        total, successful, format_duration(total_duration)
    )
}

// Pure function to format a single test result
fn format_test_result(result: &TestResult) -> String {
    format!(
        "\nTest: {}\nSuccess: {}\nDuration: {}\nStatus: {}\nError: {}",
        result.name,
        result.success,
        format_duration(result.duration),
        result.status,
        result.error.as_deref().unwrap_or("None")
    )
}

// Higher-order function to format test results
fn format_test_results<F>(format_single: F) -> impl Fn(&[TestResult]) -> String
where
    F: Fn(&TestResult) -> String,
{
    move |results| results.iter().map(&format_single).collect::<Vec<String>>().join("\n")
}

// Pure function to format all results
fn format_all_results(results: &[TestResult]) -> String {
    let (total, successful, total_duration) = analyze_results(results);
    let summary = format_summary(total, successful, total_duration);
    let detailed_results = format_test_results(format_test_result)(results);
    format!("{}\n{}", summary, detailed_results)
}

// Composition function to run API tests
pub async fn run_api_tests(api_test_path: &str) -> Result<String, AppError> {
    let load_tests = compose_read_and_parse(read_file_content, parse_and_load_tests);
    let client = Arc::new(create_client());
    let run_tests = run_tests_with_client(client);

    let tests = load_tests(Path::new(api_test_path))?;
    let results = run_tests(tests).await;
    Ok(format_all_results(&results))
}

// Higher-order function to compose multiple test runners
pub fn compose_test_runners<F, G>(
    runner1: F,
    runner2: G,
) -> impl Fn(String) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'static>>
where
    F: Fn(&str) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'static>> + Send + Sync + Clone + 'static,
    G: Fn(&str) -> Pin<Box<dyn Future<Output = Result<String, AppError>> + Send + 'static>> + Send + Sync + Clone + 'static,
{
    move |path| {
        let runner1 = runner1.clone();
        let runner2 = runner2.clone();
        let path_clone = path.clone();
        Box::pin(async move {
            let result1 = runner1(&path).await?;
            let result2 = runner2(&path_clone).await?;
            Ok(format!("{}\n{}", result1, result2))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_run_api_tests_with_sample_file() {
        // Use the existing sample file from test_examples directory
        let test_file = Path::new("test_examples/sample_restfulAPI_test.json");
        
        // Run test with the sample file
        let result = run_api_tests(test_file.to_str().unwrap()).await;
        assert!(result.is_ok(), "Failed to run API tests: {:?}", result.err());
        
        // Verify the result contains expected output
        let output = result.unwrap();
        assert!(output.contains("API Test Results"), "Output missing API Test Results");
        assert!(output.contains("Total tests:"), "Output missing test count");
        assert!(output.contains("Successful tests:"), "Output missing successful tests count");
    }
}