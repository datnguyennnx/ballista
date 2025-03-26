use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTest {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub expected_status: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTestConfig {
    pub tests: Vec<ApiTest>,
}

#[derive(Debug, Clone)]
pub struct RequestResult {
    pub status: u16,
    pub duration: Duration,
    pub error: Option<String>,
    pub json: Option<Value>,
} 