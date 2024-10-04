use reqwest::Client;
use tokio::time::Instant;
use std::sync::Arc;
use serde_json::Value;

#[derive(Debug)]
pub struct RequestResult {
    pub duration: std::time::Duration,
    pub status: u16,
    pub json: Option<Value>,
}

#[derive(Clone)]
pub struct TestConfig {
    pub urls: Arc<Vec<String>>,
    pub concurrency: u32,
    pub total_requests: u32,
    pub duration: Option<u64>,
}

pub async fn send_request(client: &Client, url: &str) -> Result<RequestResult, reqwest::Error> {
    let start = Instant::now();
    let response = client.get(url).send().await?;
    let status = response.status().as_u16();
    let json = response.json::<Value>().await.ok();
    Ok(RequestResult {
        duration: start.elapsed(),
        status,
        json,
    })
}