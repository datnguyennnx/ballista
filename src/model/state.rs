use std::sync::{Arc, Mutex, atomic::AtomicBool};
use tokio::sync::{broadcast, mpsc::Sender};
use axum::extract::ws::Message;
use uuid::Uuid;
use crate::model::test::{TestResult, TestUpdate};
use crate::model::time_series::{TimeSeriesTracker, TimeSeriesPoint};

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
    /// Active WebSocket connections
    pub ws_clients: Arc<Mutex<Vec<Sender<Message>>>>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> (Self, broadcast::Sender<String>) {
        let (tx, _) = broadcast::channel(100);
        let state = Self {
            tx: tx.clone(),
            test_results: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(AtomicBool::new(false)),
            time_series: Arc::new(Mutex::new(TimeSeriesTracker::new())),
            ws_clients: Arc::new(Mutex::new(Vec::new())),
        };
        
        (state, tx)
    }
    
    /// Add a WebSocket connection to the state
    pub fn add_ws_connection(&self, tx: Sender<Message>) {
        let mut clients = self.ws_clients.lock().unwrap();
        clients.push(tx);
    }
    
    /// Remove a WebSocket connection from the state
    pub fn remove_ws_connection(&self, tx: &Sender<Message>) {
        let mut clients = self.ws_clients.lock().unwrap();
        if let Some(pos) = clients.iter().position(|x| std::ptr::eq(x, tx)) {
            clients.remove(pos);
        }
    }
    
    /// Get all active WebSocket connections
    pub fn get_ws_connections(&self) -> Vec<Sender<Message>> {
        let clients = self.ws_clients.lock().unwrap();
        clients.clone()
    }
    
    /// Filter out closed or stale WebSocket connections to prevent errors
    pub fn clean_ws_connections(&self) {
        let mut clients = self.ws_clients.lock().unwrap();
        
        // Create a new vec to hold valid connections
        let mut valid_clients = Vec::new();
        
        // For each client, try to poll the channel capacity
        // If the channel is closed, the capacity check will return 0
        for client in clients.drain(..) {
            if client.capacity() > 0 {
                valid_clients.push(client);
            }
        }
        
        // If we found some closed connections, replace the list and log
        if valid_clients.len() < clients.len() {
            tracing::info!("Cleaned up {} stale WebSocket connections", clients.len() - valid_clients.len());
        }
        
        // Replace with the valid clients
        *clients = valid_clients;
    }
    
    /// Generate a unique test ID
    pub fn generate_test_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Add a test result to the state
    pub async fn add_test_result(&self, result: TestResult) {
        let mut results = self.test_results.lock().unwrap();
        results.push(result);
    }
    
    /// Send a test update through the broadcast channel
    pub async fn send_test_update(&self, update: TestUpdate) -> Result<(), crate::model::error::AppError> {
        // Create a TestResult from the TestUpdate fields
        let test_result = TestResult {
            id: update.id.clone(),
            test_type: update.test_type,
            status: update.status,
            metrics: update.metrics.clone(),
            error: update.error.clone(),
            timestamp: update.timestamp,
        };
        
        // Clean up stale connections before sending
        self.clean_ws_connections();
        
        // Send to WebSocket clients directly
        let clients = self.get_ws_connections();
        if !clients.is_empty() {
            // Import the send_test_update function only if we have clients
            crate::controller::websocket::send_test_update(&clients, test_result);
        }
        
        // Also continue to use broadcast channel for backward compatibility
        match serde_json::to_string(&update) {
            Ok(json) => {
                let _ = self.tx.send(json);
                Ok(())
            },
            Err(e) => Err(crate::model::error::AppError::SerializationError(e)),
        }
    }
    
    /// Update time series data and send an update
    pub async fn update_time_series(&self, metrics: &crate::model::test::TestMetrics) -> Result<(), crate::model::error::AppError> {
        // Update the time series data
        let time_series_point = {
            let mut time_series = self.time_series.lock().unwrap();
            time_series.add_point(metrics);
            time_series.get_latest_point()
        };
        
        // Send the time series point over WebSocket if available
        if let Some(point) = time_series_point {
            // Clean up stale connections before sending
            self.clean_ws_connections();
            
            // Send directly to WebSocket clients
            let clients = self.get_ws_connections();
            if !clients.is_empty() {
                // Import the send_time_series_update function only if we have clients
                crate::controller::websocket::send_time_series_update(&clients, point.clone());
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
    
    /// Get all time series points
    pub fn get_time_series_points(&self) -> Vec<crate::model::time_series::TimeSeriesPoint> {
        let time_series = self.time_series.lock().unwrap();
        time_series.get_points()
    }
    
    /// Reset time series data for a new test
    pub fn reset_time_series(&self) {
        let mut time_series = self.time_series.lock().unwrap();
        time_series.reset();
    }
    
    /// Get all test results
    pub async fn get_all_test_results(&self) -> Vec<TestResult> {
        let results = self.test_results.lock().unwrap();
        results.clone()
    }
    
    /// Get a test result by ID
    pub async fn get_test_result(&self, id: &str) -> Option<TestResult> {
        let results = self.test_results.lock().unwrap();
        results.iter()
            .find(|r| r.id == id)
            .cloned()
    }
} 