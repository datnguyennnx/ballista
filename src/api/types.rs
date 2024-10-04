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