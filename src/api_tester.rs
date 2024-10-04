use serde_json::Value;
use reqwest::{Client, Error as ReqwestError, StatusCode};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::Instant;
use futures::future::join_all;
use async_trait::async_trait;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiTest {
    name: String,
    url: String,
    method: String,
    headers: Option<HashMap<String, String>>,
    body: Option<Value>,
    expected_status: u16,
    expected_body: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub success: bool,
    pub duration: Duration,
    pub status: u16,
    pub error: Option<String>,
}

#[derive(Debug)]
pub enum ApiTestError {
    UnsupportedMethod(String),
    ReqwestError(ReqwestError),
}

impl From<ReqwestError> for ApiTestError {
    fn from(error: ReqwestError) -> Self {
        ApiTestError::ReqwestError(error)
    }
}

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

pub fn analyze_results(results: &[TestResult]) -> (usize, usize, Duration) {
    let total = results.len();
    let successful = results.iter().filter(|r| r.success).count();
    let total_duration = results.iter().map(|r| r.duration).sum();
    (total, successful, total_duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        pub MockHttpClient {}
        #[async_trait]
        impl HttpClient for MockHttpClient {
            async fn send_request(&self, test: &ApiTest) -> Result<reqwest::Response, ApiTestError>;
        }
    }

    #[tokio::test]
    async fn test_run_test() {
        let mut mock_client = MockHttpClient::new();
        mock_client.expect_send_request()
            .returning(|_| {
                Ok(reqwest::Response::from(
                    http::Response::builder()
                        .status(200)
                        .body("{}")
                        .unwrap()
                ))
            });

        let test = ApiTest {
            name: "Test GET".to_string(),
            url: "https://example.com".to_string(),
            method: "GET".to_string(),
            headers: None,
            body: None,
            expected_status: 200,
            expected_body: None,
        };

        let result = run_test(&mock_client, &test).await;
        assert!(result.success);
    }

    #[test]
    fn test_load_tests_from_json() {
        let json_str = r#"
        [
            {
                "name": "Test GET request",
                "url": "https://jsonplaceholder.typicode.com/todos/1",
                "method": "GET",
                "expected_status": 200
            }
        ]
        "#;

        let tests = load_tests_from_json(json_str).unwrap();
        assert_eq!(tests.len(), 1);
        assert_eq!(tests[0].name, "Test GET request");
    }

    #[test]
    fn test_analyze_results() {
        let results = vec![
            TestResult {
                name: "Test 1".to_string(),
                success: true,
                duration: Duration::from_secs(1),
                status: 200,
                error: None,
            },
            TestResult {
                name: "Test 2".to_string(),
                success: false,
                duration: Duration::from_secs(2),
                status: 404,
                error: Some("Not found".to_string()),
            },
        ];

        let (total, successful, total_duration) = analyze_results(&results);
        assert_eq!(total, 2);
        assert_eq!(successful, 1);
        assert_eq!(total_duration, Duration::from_secs(3));
    }
}