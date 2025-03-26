use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestConfig {
    pub target_url: String,
    pub concurrent_users: u32,
    pub duration_secs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub target_url: String,
    pub concurrent_users: u32,
    pub duration_secs: u32,
    pub num_requests: u32,
}

pub fn create_test_config_from_stress(config: &StressTestConfig) -> TestConfig {
    TestConfig {
        target_url: config.target_url.clone(),
        concurrent_users: config.concurrent_users,
        duration_secs: config.duration_secs,
        num_requests: 0,
    }
} 