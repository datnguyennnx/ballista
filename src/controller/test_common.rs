use axum::{
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use crate::model::state::AppState;
use crate::model::test::{TestType, TestStatus, TestResult, TestMetrics, TestUpdate};

/// Common test context for managing test state and updates
pub struct TestContext {
    state: Arc<AppState>,
    test_type: TestType,
    test_id: String,
}

impl TestContext {
    /// Create a new test context
    pub async fn new(
        state: Arc<AppState>,
        test_type: TestType,
    ) -> Result<(Self, Response), Response> {
        let test_id = state.generate_test_id();
        
        // Create initial test result
        let result = TestResult {
            id: test_id.clone(),
            test_type,
            status: TestStatus::Pending,
            progress: 0.0,
            metrics: None,
            error: None,
            start_time: chrono::Utc::now(),
            end_time: None,
        };
        
        // Add to state
        state.add_test_result(result).await;
        
        // Reset time series for new test
        state.reset_time_series().await;
        
        Ok((
            Self {
                state,
                test_type,
                test_id: test_id.clone(),
            },
            Json(serde_json::json!({
                "id": test_id,
                "status": "started"
            })).into_response(),
        ))
    }

    /// Get the test ID associated with this context
    pub fn test_id(&self) -> &str {
        &self.test_id
    }
    
    /// Send a test update
    pub async fn send_update(
        &self,
        status: TestStatus,
        progress: f32,
        metrics: Option<TestMetrics>,
        error: Option<String>,
    ) {
        let update = TestUpdate {
            id: self.test_id.clone(),
            test_type: self.test_type,
            status,
            progress,
            metrics: metrics.clone(),
            error: error.clone(),
        };
        
        // Send update through broadcast channel
        if let Err(e) = self.state.send_test_update(update).await {
            tracing::error!("Failed to send test update for {}: {}", self.test_id, e); // Added test_id to log
        }
        
        // Update test result in state
        if let Some(mut result) = self.state.get_test_result(&self.test_id).await {
            result.status = status;
            result.progress = progress;
            result.metrics = metrics;
            result.error = error;
            if status == TestStatus::Completed || status == TestStatus::Error {
                result.end_time = Some(chrono::Utc::now());
            }
            // Use update_test_result method if available, otherwise add (which might duplicate)
            // Assuming add_test_result handles updates correctly (e.g., replaces based on ID)
            self.state.add_test_result(result).await;
        } else {
             tracing::warn!("Could not find test result {} to update state.", self.test_id);
        }
    }
    
    /// Complete a test
    pub async fn complete_test(&self, metrics: TestMetrics, error: Option<String>) {
        let final_status = if error.is_some() { TestStatus::Error } else { TestStatus::Completed };
        tracing::info!("Completing test {} with status: {:?}", self.test_id, final_status); // Added logging
        self.send_update(
            final_status,
            100.0,
            Some(metrics),
            error,
        ).await;
    }

    /// Update time series data
    pub async fn update_time_series(&self, metrics: &TestMetrics) -> Result<(), crate::model::error::AppError> {
        self.state.update_time_series(metrics).await
    }
}