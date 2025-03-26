// This file is kept for backward compatibility
// Future code should use the functions from http/client.rs directly

pub use crate::model::test::RequestResult;
use crate::model::test::TestConfig;

// Legacy TestConfig struct kept for backward compatibility
#[derive(Clone)]
pub struct LegacyTestConfig {
    pub urls: Vec<String>,
    pub concurrency: u32,
    pub total_requests: Option<u32>,
    pub duration: Option<u64>,
}

// Conversion function from legacy to new config
pub fn convert_legacy_config(config: &LegacyTestConfig) -> TestConfig {
    TestConfig {
        target_url: config.urls.first().unwrap_or(&String::new()).clone(),
        concurrent_users: config.concurrency,
        duration_secs: config.duration.unwrap_or(0) as u32,
        num_requests: config.total_requests.unwrap_or(0) as u32,
    }
}

#[derive(Debug, Clone)]
pub struct RequestConfig {
    pub target_url: String,
    pub concurrent_users: u32,
    pub duration_secs: u32,
    pub num_requests: u32,
}

impl From<&TestConfig> for RequestConfig {
    fn from(config: &TestConfig) -> Self {
        Self {
            target_url: config.target_url.clone(),
            concurrent_users: config.concurrent_users,
            duration_secs: config.duration_secs,
            num_requests: config.num_requests,
        }
    }
}