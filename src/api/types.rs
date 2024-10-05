use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiTest {
    pub name: String,
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Value>,
    pub expected_status: u16,
    pub expected_body: Option<Value>,
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
    ReqwestError(reqwest::Error),
}

impl From<reqwest::Error> for ApiTestError {
    fn from(error: reqwest::Error) -> Self {
        ApiTestError::ReqwestError(error)
    }
}

// Pure function to create a new TestResult
pub fn create_test_result(
    name: String,
    success: bool,
    duration: Duration,
    status: u16,
    error: Option<String>,
) -> TestResult {
    TestResult {
        name,
        success,
        duration,
        status,
        error,
    }
}

// Pure function to check if a test is successful
pub fn is_test_successful(test: &ApiTest, status: u16, body: &Option<Value>) -> bool {
    status == test.expected_status && 
    (test.expected_body.is_none() || body == &test.expected_body)
}