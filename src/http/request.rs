// This file is kept for backward compatibility
// Future code should use the functions from http/client.rs directly

pub use crate::model::test::RequestResult;

// Legacy TestConfig struct kept for backward compatibility
#[derive(Clone)]
pub struct LegacyTestConfig {
    pub urls: Vec<String>,
    pub concurrency: u32,
    pub total_requests: Option<u32>,
    pub duration: Option<u64>,
}

// Conversion function from legacy to new config
pub fn convert_legacy_config(config: &LegacyTestConfig) -> crate::model::test::TestConfig {
    crate::model::test::TestConfig {
        urls: config.urls.clone(),
        concurrency: config.concurrency,
        total_requests: config.total_requests,
        duration: config.duration,
    }
}