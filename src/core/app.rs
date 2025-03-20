use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    Load,
    Stress,
    Api,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    InProgress,
    Success,
    Failure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_type: TestType,
    pub status: TestStatus,
    pub details: String,
    pub timestamp: DateTime<Utc>,
}

pub struct AppState {
    test_results: Mutex<Vec<TestResult>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            test_results: Mutex::new(Vec::new()),
        }
    }

    pub fn add_test_result(&self, result: TestResult) {
        let mut results = self.test_results.lock().unwrap();
        results.push(result);
    }

    pub fn get_test_results(&self) -> Vec<TestResult> {
        let results = self.test_results.lock().unwrap();
        results.clone()
    }
} 