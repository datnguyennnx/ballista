use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use std::sync::Arc;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use crate::router::AppState;
use crate::router::test::{TestConfig, TestUpdate, TestType, TestStatus};
use chrono;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to broadcast
    let mut rx = state.tx.subscribe();

    // Handle incoming messages
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle received messages (if needed)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                _ => continue,
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

// Pure function to process test requests
async fn process_test_request(
    test_config: &str,
    tx: &Arc<broadcast::Sender<TestUpdate>>,
) -> Result<(), serde_json::Error> {
    let config: TestConfig = serde_json::from_str(test_config)?;
    
    // Create a unique test ID
    let test_id = uuid::Uuid::new_v4().to_string();
    
    // Spawn the test execution
    tokio::spawn(execute_test(config, test_id, tx.clone()));
    
    Ok(())
}

// Pure function to execute tests
async fn execute_test(
    config: TestConfig,
    test_id: String,
    tx: Arc<broadcast::Sender<TestUpdate>>,
) {
    // Initial update
    let _ = tx.send(TestUpdate {
        id: test_id.clone(),
        test_type: TestType::Load, // Default to Load test type
        status: TestStatus::Started,
        progress: 0.0,
        metrics: None,
        error: None,
        timestamp: chrono::Utc::now().timestamp(),
    });

    // Test execution logic here...
    // This would be implemented based on your testing requirements
} 