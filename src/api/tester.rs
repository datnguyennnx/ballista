use crate::api::types::{ApiTest, TestResult, ApiTestError, create_test_result, is_test_successful};
use futures::future::{self, Future};
use reqwest::{Client, StatusCode, Response, RequestBuilder};
use serde_json::Value;
use tokio::time::{Instant, Duration};

// Pure function to create a request based on the ApiTest
fn create_request(client: &Client, test: &ApiTest) -> Result<RequestBuilder, ApiTestError> {
    let create_method = |method: &str| match method.to_lowercase().as_str() {
        "get" => Ok(client.get(&test.url)),
        "post" => Ok(client.post(&test.url)),
        "put" => Ok(client.put(&test.url)),
        "delete" => Ok(client.delete(&test.url)),
        _ => Err(ApiTestError::UnsupportedMethod(method.to_string())),
    };

    let add_headers = |mut req: RequestBuilder| {
        if let Some(headers) = &test.headers {
            for (key, value) in headers {
                req = req.header(key, value);
            }
        }
        req
    };

    let add_body = |mut req: RequestBuilder| {
        if let Some(body) = &test.body {
            req = req.json(body);
        }
        req
    };

    create_method(&test.method)
        .map(add_headers)
        .map(add_body)
}

// Pure function to process the response
async fn process_response(response: Response, test: &ApiTest, duration: Duration) -> TestResult {
    let status = response.status().as_u16();
    let body = response.json::<Value>().await.ok();
    let success = is_test_successful(test, status, &body);

    create_test_result(
        test.name.clone(),
        success,
        duration,
        status,
        None,
    )
}

// Pure function to handle errors
fn handle_error(error: ApiTestError, test: &ApiTest, duration: Duration) -> TestResult {
    let (status, error_message) = match &error {
        ApiTestError::UnsupportedMethod(m) => (StatusCode::METHOD_NOT_ALLOWED.as_u16(), format!("Unsupported HTTP method: {}", m)),
        ApiTestError::ReqwestError(e) => (e.status().map_or(0, |s| s.as_u16()), e.to_string()),
    };

    create_test_result(
        test.name.clone(),
        false,
        duration,
        status,
        Some(error_message),
    )
}

// Higher-order function to measure execution time
async fn with_timing<F, Fut, T>(f: F) -> (Duration, T)
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    let start = Instant::now();
    let result = f().await;
    let duration = start.elapsed();
    (duration, result)
}

// Main function to run a single test
pub async fn run_test(client: &Client, test: &ApiTest) -> TestResult {
    let (duration, result) = with_timing(|| async {
        match create_request(client, test) {
            Ok(req) => req.send().await.map_err(ApiTestError::from),
            Err(e) => Err(e),
        }
    }).await;

    match result {
        Ok(response) => process_response(response, test, duration).await,
        Err(error) => handle_error(error, test, duration),
    }
}

// Function to run multiple tests
pub async fn run_tests(client: &Client, tests: &[ApiTest]) -> Vec<TestResult> {
    future::join_all(tests.iter().map(|test| run_test(client, test))).await
}

// Pure function to load tests from JSON
pub fn load_tests_from_json(json_str: &str) -> Result<Vec<ApiTest>, serde_json::Error> {
    serde_json::from_str(json_str)
}

// Pure function to analyze results
pub fn analyze_results(results: &[TestResult]) -> (usize, usize, Duration) {
    let total = results.len();
    let successful = results.iter().filter(|r| r.success).count();
    let total_duration = results.iter().fold(Duration::ZERO, |acc, r| acc + r.duration);
    (total, successful, total_duration)
}

// Higher-order function to apply a transformation to all test results
pub fn map_results<F, T>(results: &[TestResult], f: F) -> Vec<T>
where
    F: Fn(&TestResult) -> T,
{
    results.iter().map(f).collect()
}

// Higher-order function to filter test results
pub fn filter_results<F>(results: &[TestResult], predicate: F) -> Vec<&TestResult>
where
    F: Fn(&TestResult) -> bool,
{
    results.iter().filter(|&r| predicate(r)).collect()
}