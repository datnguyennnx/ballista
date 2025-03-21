use std::sync::{Arc, Mutex, atomic::AtomicBool};
use tokio::sync::broadcast;
use uuid::Uuid;
use crate::model::test::{TestResult, TestUpdate};

/// Application state shared across request handlers
#[derive(Clone)]
pub struct AppState {
    /// Channel for real-time updates
    pub tx: broadcast::Sender<String>,
    /// Track test results
    pub test_results: Arc<Mutex<Vec<TestResult>>>,
    /// Flag to track if a test is running
    pub is_running: Arc<AtomicBool>,
}

impl AppState {
    /// Create a new application state
    pub fn new() -> (Self, broadcast::Sender<String>) {
        let (tx, _) = broadcast::channel(100);
        let state = Self {
            tx: tx.clone(),
            test_results: Arc::new(Mutex::new(Vec::new())),
            is_running: Arc::new(AtomicBool::new(false)),
        };
        
        (state, tx)
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
        match serde_json::to_string(&update) {
            Ok(json) => {
                let _ = self.tx.send(json);
                Ok(())
            },
            Err(e) => Err(crate::model::error::AppError::SerializationError(e)),
        }
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