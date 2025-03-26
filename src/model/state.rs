use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::{broadcast, Mutex, mpsc::{Sender, error::TrySendError}};
use axum::extract::ws::Message;
use crate::model::test::{TestResult, TestUpdate};
use crate::model::time_series::TimeSeriesTracker;
use rand::Rng;
use serde_json::json;
use tracing::{info, warn, error};

const CHANNEL_SIZE: usize = 1024;

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    /// Channel for real-time updates
    pub tx: broadcast::Sender<String>,
    /// Track test results
    pub test_results: Arc<Mutex<Vec<TestResult>>>,
    /// Flag to track if a test is running
    pub is_running: Arc<AtomicBool>,
    /// Time series data tracker
    pub time_series: Arc<Mutex<TimeSeriesTracker>>,
    /// Active WebSocket connection
    pub ws_client: Arc<Mutex<Option<Sender<Message>>>>,
    /// Channel for test updates
    pub test_updates: broadcast::Sender<TestUpdate>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> (Self, broadcast::Sender<String>) {
        let (tx, _) = broadcast::channel(CHANNEL_SIZE);
        let (test_updates, _) = broadcast::channel(CHANNEL_SIZE);
        let state = Self {
            tx: tx.clone(),
            test_results: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            time_series: Arc::new(Mutex::new(TimeSeriesTracker::new())),
            ws_client: Arc::new(Mutex::new(None)),
            test_updates,
        };
        
        (state, tx)
    }
    
    /// Set the active WebSocket connection
    pub async fn set_ws_connection(&self, tx: Sender<Message>) -> bool {
        let mut client = self.ws_client.lock().await;
        
        // Check if existing connection is still valid
        if let Some(existing) = client.as_ref() {
            if existing.capacity() > 0 {
                match existing.try_send(Message::Ping(vec![])) {
                    Ok(_) => {
                        warn!("Active WebSocket connection exists, rejecting new connection");
                        return false;
                    },
                    Err(_) => {
                        info!("Existing connection is stale, replacing");
                    }
                }
            }
        }
        
        *client = Some(tx);
        true
    }
    
    /// Remove the WebSocket connection
    pub async fn remove_ws_connection(&self) {
        let mut client = self.ws_client.lock().await;
        *client = None;
    }
    
    /// Get the active WebSocket connection if it exists
    pub async fn get_ws_connection(&self) -> Option<Sender<Message>> {
        let client = self.ws_client.lock().await;
        client.clone()
    }
    
    /// Send a test update through the broadcast channel and WebSocket
    pub async fn send_test_update(&self, update: TestUpdate) -> Result<(), String> {
        // First try WebSocket
        if let Some(client) = self.get_ws_connection().await {
            let msg = json!({
                "type": "test_update",
                "data": update.clone()
            });
            
            if let Ok(json) = serde_json::to_string(&msg) {
                match client.try_send(Message::Text(json)) {
                    Ok(_) => {
                        // info!("Test update sent via WebSocket for", update.id);
                    },
                    Err(e) => match e {
                        TrySendError::Full(_) => {
                            // warn!("Client message queue is full, dropping message for", update.id);
                        },
                        TrySendError::Closed(_) => {
                            // info!("WebSocket connection closed, removing for", update.id);
                            self.remove_ws_connection().await;
                        }
                    }
                }
            }
        }

        // Then try broadcast channel
        match self.test_updates.send(update.clone()) {
            Ok(_) => {
                info!("Test update sent via broadcast for test-{}", update.id);
                Ok(())
            },
            Err(e) => {
                // If send failed, try to send through WebSocket only
                if let Some(client) = self.get_ws_connection().await {
                    let msg = json!({
                        "type": "test_update",
                        "data": update
                    });
                    
                    if let Ok(json) = serde_json::to_string(&msg) {
                        match client.try_send(Message::Text(json)) {
                            Ok(_) => {
                                // info!("Test update sent via WebSocket fallback for test-{}", update.id);
                                Ok(())
                            },
                            Err(_) => {
                                error!("Failed to send test update via any channel for test-{}", update.id);
                                Err(format!("Failed to send test update: {}", e))
                            }
                        }
                    } else {
                        Err(format!("Failed to serialize test update: {}", e))
                    }
                } else {
                    error!("No available channels to send test update for test-{}", update.id);
                    Err(format!("Failed to send test update: {}", e))
                }
            }
        }
    }
    
    /// Update time series data and send an update
    pub async fn update_time_series(&self, metrics: &crate::model::test::TestMetrics) -> Result<(), crate::model::error::AppError> {
        // Update the time series data
        let time_series = self.time_series.lock().await;
        time_series.add_point(metrics).await;
        
        // Get the latest point
        let points = time_series.get_points().await;
        if let Some(point) = points.last() {
            // Send to WebSocket client if connected
            if let Some(client) = self.get_ws_connection().await {
                let msg = json!({
                    "type": "time_series_update",
                    "data": point
                });
                
                if let Ok(json) = serde_json::to_string(&msg) {
                    match client.try_send(Message::Text(json)) {
                        Ok(_) => {},
                        Err(e) => match e {
                            TrySendError::Full(_) => {
                                warn!("Client message queue is full");
                            },
                            TrySendError::Closed(_) => {
                                info!("WebSocket connection closed, removing");
                                self.remove_ws_connection().await;
                            }
                        }
                    }
                }
            }
            
            // Also continue to use broadcast channel for backward compatibility
            match serde_json::to_string(&point) {
                Ok(json) => {
                    let _ = self.tx.send(json);
                    Ok(())
                },
                Err(e) => Err(crate::model::error::AppError::SerializationError(e)),
            }
        } else {
            Ok(())
        }
    }
    
    /// Generate a unique test ID
    pub fn generate_test_id(&self) -> String {
        let mut rng = rand::thread_rng();
        let id: u32 = rng.gen();
        format!("test-{}", id)
    }
    
    /// Add a test result to the state
    pub async fn add_test_result(&self, result: TestResult) {
        let mut results = self.test_results.lock().await;
        results.push(result);
    }
    
    /// Get all time series points
    pub async fn get_time_series_points(&self) -> Vec<crate::model::time_series::TimeSeriesPoint> {
        let time_series = self.time_series.lock().await;
        time_series.get_points().await
    }
    
    /// Reset time series data for a new test
    pub async fn reset_time_series(&self) {
        let time_series = self.time_series.lock().await;
        time_series.reset().await;
    }
    
    /// Get all test results
    pub async fn get_all_test_results(&self) -> Vec<TestResult> {
        let results = self.test_results.lock().await;
        results.clone()
    }
    
    /// Get a test result by ID
    pub async fn get_test_result(&self, id: &str) -> Option<TestResult> {
        let results = self.test_results.lock().await;
        results.iter()
            .find(|r| r.id == id)
            .cloned()
    }
} 