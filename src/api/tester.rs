use crate::api::types::{ApiTest, TestResult, ApiTestError};
use async_trait::async_trait;
use futures::future::join_all;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use tokio::time::Instant;

#[async_trait]
pub trait HttpClient {
    async fn send_request(&self, test: &ApiTest) -> Result<reqwest::Response, ApiTestError>;
}

#[derive(Clone)]
pub struct ReqwestClient(Client);

impl ReqwestClient {
    pub fn new() -> Self {
        ReqwestClient(Client::new())
    }
}

#[async_trait]
impl HttpClient for ReqwestClient {
    async fn send_request(&self, test: &ApiTest) -> Result<reqwest::Response, ApiTestError> {
        let mut request = match test.method.to_lowercase().as_str() {
            "get" => self.0.get(&test.url),
            "post" => self.0.post(&test.url),
            "put" => self.0.put(&test.url),
            "delete" => self.0.delete(&test.url),
            _ => return Err(ApiTestError::UnsupportedMethod(test.method.clone())),
        };

        if let Some(headers) = &test.headers {
            for (key, value) in headers {
                request = request.header(key, value);
            }
        }

        if let Some(body) = &test.body {
            request = request.json(body);
        }

        Ok(request.send().await?)
    }
}

pub async fn run_test<T: HttpClient>(client: &T, test: &ApiTest) -> TestResult {
    let start = Instant::now();
    
    match client.send_request(test).await {
        Ok(response) => {
            let status = response.status().as_u16();
            let body = response.json::<Value>().await.ok();
            let duration = start.elapsed();
            let success = status == test.expected_status && 
                          (test.expected_body.is_none() || body == test.expected_body);

            TestResult {
                name: test.name.clone(),
                success,
                duration,
                status,
                error: None,
            }
        },
        Err(e) => TestResult {
            name: test.name.clone(),
            success: false,
            duration: start.elapsed(),
            status: match e {
                ApiTestError::UnsupportedMethod(_) => StatusCode::METHOD_NOT_ALLOWED.as_u16(),
                ApiTestError::ReqwestError(ref e) => e.status().map_or(0, |s| s.as_u16()),
            },
            error: Some(match e {
                ApiTestError::UnsupportedMethod(m) => format!("Unsupported HTTP method: {}", m),
                ApiTestError::ReqwestError(e) => e.to_string(),
            }),
        },
    }
}

pub async fn run_tests<T: HttpClient>(client: &T, tests: &[ApiTest]) -> Vec<TestResult> {
    join_all(tests.iter().map(|test| run_test(client, test))).await
}

pub fn load_tests_from_json(json_str: &str) -> Result<Vec<ApiTest>, serde_json::Error> {
    serde_json::from_str(json_str)
}

pub fn analyze_results(results: &[TestResult]) -> (usize, usize, std::time::Duration) {
    let total = results.len();
    let successful = results.iter().filter(|r| r.success).count();
    let total_duration = results.iter().map(|r| r.duration).sum();
    (total, successful, total_duration)
}