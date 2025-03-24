use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::mpsc::{self, Sender};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info, warn};

use crate::model::state::AppState;
use crate::model::test::TestResult;
use crate::model::time_series::TimeSeriesPoint;

// WebSocket configuration constants
const CLIENT_TIMEOUT_SECS: u64 = 5;
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(600); // Increased timeout to 10 minutes

/// WebSocket handler that upgrades the connection and forwards to the handle_socket function
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::info!("WebSocket connection request received");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle the WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    // Clone the state for the receiver task
    let state_clone = Arc::clone(&state);
    
    // Create a channel to forward test updates to all clients
    let (tx, mut rx) = mpsc::channel(100);

    // Track active connections
    state.add_ws_connection(tx.clone());
    info!("WebSocket connection established");

    // Split the socket
    let (mut sender, mut receiver) = socket.split();

    // Track last activity time for timeout
    let mut last_activity = Instant::now();

    // Send initial time series data to client
    let time_series_points = state.get_time_series_points();
    if !time_series_points.is_empty() {
        let msg = json!({
            "type": "time_series_history",
            "data": time_series_points
        })
        .to_string();
        
        // Add timeout to send operation to prevent blocking indefinitely
        if let Err(e) = timeout(Duration::from_secs(CLIENT_TIMEOUT_SECS), sender.send(Message::Text(msg))).await {
            // If sending fails, we'll just continue - don't terminate the connection
            error!("Failed to send initial time series data: {}", e);
        }
    }

    // Clone TX for the receiver task to avoid moving ownership
    let tx_for_receiver = tx.clone();

    // Spawn a task for receiving messages
    let receive_task = tokio::spawn(async move {
        let mut interval_counter = 0u64;
        
        while let Some(result) = receiver.next().await {
            // Update activity timestamp for every message
            last_activity = Instant::now();
            
            match result {
                Ok(Message::Text(text)) => {
                    // Process text messages from client
                    process_client_message(text, &state_clone, &tx_for_receiver).await;
                }
                Ok(Message::Ping(data)) => {
                    // Respond to ping with pong
                    if let Err(e) = tx_for_receiver.send(Message::Pong(data)).await {
                        // Only log errors that aren't related to channel closure
                        if !e.to_string().contains("channel closed") {
                            error!("Failed to send pong: {}", e);
                        }
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("Client initiated close");
                    break;
                }
                Ok(Message::Binary(_)) => {
                    warn!("Unexpected binary message from client");
                }
                Ok(Message::Pong(_)) => {
                    // Standard pong response, just update activity time
                }
                Err(e) => {
                    // This is expected when browsers navigate away or refresh
                    if e.to_string().contains("connection reset") || e.to_string().contains("protocol error") {
                        info!("Client disconnected: normal browser navigation or refresh");
                    } else {
                        error!("Error receiving message: {}", e);
                    }
                    break;
                }
            }

            // Check for client timeout (only if no activity for a long time)
            if last_activity.elapsed() > CONNECTION_TIMEOUT {
                warn!("Client connection timed out after inactivity");
                break;
            }

            // Rate limiting check - avoid processing too many messages at once
            interval_counter += 1;
            if interval_counter % 10 == 0 {
                sleep(Duration::from_millis(1)).await;
            }
        }
        
        info!("WebSocket receiver task completed");
    });

    // Create a handle we can use to abort the receive task
    let receive_task_handle = receive_task.abort_handle();

    // Spawn a task for sending messages
    let send_task = tokio::spawn(async move {
        // Use a mutable last_activity variable local to this task
        let mut task_last_activity = last_activity;
        
        loop {
            tokio::select! {
                // Process messages from the channel
                Some(msg) = rx.recv() => {
                    // Try to send the message, but don't crash if it fails
                    if let Err(e) = sender.send(msg).await {
                        // Only log errors that aren't related to normal browser behavior
                        if !e.to_string().contains("connection reset") &&
                           !e.to_string().contains("broken pipe") &&
                           !e.to_string().contains("protocol error") {
                            error!("Failed to send message: {}", e);
                        } else {
                            debug!("Client disconnected: {}", e);
                        }
                        break;
                    }
                    task_last_activity = Instant::now();
                }
                
                // Check for overall inactivity timeout
                _ = sleep(Duration::from_secs(30)) => {
                    if task_last_activity.elapsed() > Duration::from_secs(300) {
                        info!("Connection inactive for 5 minutes, closing");
                        // Try to send a close message, but ignore errors
                        let _ = sender.send(Message::Close(None)).await;
                        break;
                    }
                }
            }
        }
        
        // When the sender task completes, abort the receiver task
        receive_task_handle.abort();
        info!("WebSocket sender task completed");
    });

    // Wait for either task to complete
    tokio::select! {
        _ = receive_task => {
            debug!("Receive task completed first");
        }
        _ = send_task => {
            debug!("Send task completed first");
        }
    }

    // Clean up: Remove the connection from active connections
    // This uses a separate block to ensure the mutex is released quickly
    {
        state.remove_ws_connection(&tx);
    }
    
    // Trigger a cleanup of stale connections
    // Clone the state again to use in the new task
    let cleanup_state = Arc::clone(&state);
    tokio::spawn(async move {
        cleanup_state.clean_ws_connections();
    });
    
    info!("WebSocket connection closed and cleaned up");
}

// Process client messages
async fn process_client_message(text: String, state: &Arc<AppState>, tx: &Sender<Message>) {
    match text.as_str() {
        "get_time_series" => {
            debug!("Client requested time series history");
            let time_series_points = state.get_time_series_points();
            let msg = json!({
                "type": "time_series_history",
                "data": time_series_points
            })
            .to_string();
            
            if let Err(e) = tx.send(Message::Text(msg)).await {
                error!("Failed to send time series history: {}", e);
            }
        }
        _ => {
            // Try to parse as JSON for other message types
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                debug!("Received JSON message: {}", value);
                // Process structured messages here if needed
            } else {
                debug!("Unknown message from client: {}", text);
            }
        }
    }
}

// Send a test update to all connected WebSocket clients
pub fn send_test_update(tx: &[Sender<Message>], test_result: TestResult) {
    let message = json!({
        "type": "test_update",
        "data": test_result
    })
    .to_string();

    for client in tx {
        let msg_clone = message.clone();
        let client_clone = client.clone();
        
        tokio::spawn(async move {
            if let Err(e) = client_clone.send(Message::Text(msg_clone)).await {
                // Don't log channel closed errors to reduce noise
                if !e.to_string().contains("channel closed") {
                    error!("Failed to send test update: {}", e);
                }
            }
        });
    }
}

// Send a time series update to all connected WebSocket clients
pub fn send_time_series_update(tx: &[Sender<Message>], point: TimeSeriesPoint) {
    let message = json!({
        "type": "time_series",
        "data": point
    })
    .to_string();

    for client in tx {
        let msg_clone = message.clone();
        let client_clone = client.clone();
        
        tokio::spawn(async move {
            if let Err(e) = client_clone.send(Message::Text(msg_clone)).await {
                // Don't log channel closed errors to reduce noise
                if !e.to_string().contains("channel closed") {
                    error!("Failed to send time series update: {}", e);
                }
            }
        });
    }
} 