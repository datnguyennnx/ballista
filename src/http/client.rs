use reqwest::{Client, Method};
use std::sync::Arc;
use tokio::sync::mpsc;
use std::sync::atomic::{AtomicBool, Ordering};
use futures::{stream, StreamExt, Stream};
use std::time::Duration;
use anyhow::{Context, Result};
use std::pin::Pin;

use crate::model::error::AppError;
use crate::model::test::{TestConfig};
use crate::model::test::{ApiTest, RequestResult, ApiRequestResult};

// string_to_method remains the same
pub fn string_to_method(method: &str) -> Result<Method> {
    match method.to_uppercase().as_str() {
        "GET" => Ok(Method::GET),
        "POST" => Ok(Method::POST),
        "PUT" => Ok(Method::PUT),
        "DELETE" => Ok(Method::DELETE),
        "PATCH" => Ok(Method::PATCH),
        "HEAD" => Ok(Method::HEAD),
        "OPTIONS" => Ok(Method::OPTIONS),
        _ => Err(anyhow::anyhow!("Invalid HTTP method: {}", method)),
    }
}

// create_optimized_client remains the same
pub fn create_optimized_client() -> Client {
    Client::builder()
        .pool_max_idle_per_host(10)
        .tcp_keepalive(Some(Duration::from_secs(60)))
        .tcp_nodelay(true)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

// send_request remains the same
pub async fn send_request(client: &Client, url: &str) -> Result<RequestResult> {
    let start_time = std::time::Instant::now();
    let response = client.get(url).send()
        .await
        .with_context(|| format!("Failed to send request to {}", url))?;

    let status = response.status().as_u16();
    let duration = start_time.elapsed();
    let _ = response.bytes().await.context("Failed to read response body")?;

    Ok(RequestResult {
        status,
        duration,
    })
}


// send_api_request remains the same
pub async fn send_api_request(client: &Client, test: &ApiTest) -> Result<ApiRequestResult> {
    let start_time = std::time::Instant::now();

    let method = string_to_method(&test.method)?;
    let mut request_builder = client.request(method, &test.url);

    if let Some(headers) = &test.headers {
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }
    }

    if let Some(body) = &test.body {
        request_builder = request_builder.body(body.clone());
    }

    let response = request_builder.send()
        .await
        .with_context(|| format!("Failed to send API request to {}", test.url))?;

    let duration = start_time.elapsed();
    let status = response.status().as_u16();

    let json_body = if response.status().is_success() {
         response.json::<serde_json::Value>().await.ok()
    } else {
         let _ = response.text().await.context("Failed to read error response body")?;
         None
    };

    Ok(ApiRequestResult {
        status,
        duration,
        json: json_body,
    })
}


// load_test remains the same
pub async fn load_test(
    client: &Client,
    config: &TestConfig,
    result_sender: mpsc::Sender<Result<RequestResult>>,
    is_finished: Arc<AtomicBool>,
) -> Result<(), AppError> {
    tracing::info!("Starting load test: {} requests, {} concurrent users",
        config.num_requests, config.concurrent_users);
    perform_test(client, config, result_sender, is_finished)
        .await
        .map_err(|e| AppError::TestExecutionError(format!("Load test execution failed: {}", e)))
}

// stress_test remains the same
pub async fn stress_test(
    client: &Client,
    config: &TestConfig,
    result_sender: mpsc::Sender<Result<RequestResult>>,
    is_finished: Arc<AtomicBool>,
) -> Result<(), AppError> {
    tracing::info!("Starting stress test: {} seconds, {} concurrent users",
        config.duration_secs, config.concurrent_users);

    let end_time = tokio::time::Instant::now() + std::time::Duration::from_secs(config.duration_secs as u64);
    let result_sender_clone = result_sender.clone();

    let test_result = tokio::select! {
        res = perform_test(client, config, result_sender_clone, Arc::clone(&is_finished)) => res,
        _ = tokio::time::sleep_until(end_time) => {
            tracing::info!("Stress test duration reached");
            is_finished.store(true, Ordering::SeqCst);
            Ok(())
        }
    };
    drop(result_sender);
    test_result.map_err(|e| AppError::TestExecutionError(format!("Stress test execution failed: {}", e)))
}

// perform_test updated take_while closure
async fn perform_test(
    client: &Client,
    config: &TestConfig,
    result_sender: mpsc::Sender<Result<RequestResult>>,
    is_finished: Arc<AtomicBool>,
) -> Result<()> {
    let stream_iter: Pin<Box<dyn Stream<Item = ()> + Send>> = if config.num_requests > 0 {
        Box::pin(stream::iter(std::iter::repeat(()).take(config.num_requests as usize)))
    } else {
        Box::pin(stream::iter(std::iter::repeat(())))
    };

    stream_iter
        .map(|_| {
            let url = config.target_url.clone();
            let client = client.clone();
            let sender = result_sender.clone();
            async move {
                let result = send_request(&client, &url).await;
                sender.send(result).await.is_ok()
            }
        })
        .buffer_unordered(config.concurrent_users as usize)
        .take_while(|send_success| {
            let success = *send_success;
            let finished = is_finished.load(Ordering::SeqCst);
            async move { success && !finished }
        })
        .for_each(|_| async {})
        .await;

    tracing::info!("perform_test finished and dropped sender.");
    Ok(())
}