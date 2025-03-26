use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tracing::{info, warn};
use futures::{SinkExt, StreamExt};
use serde_json::json;

use crate::model::state::AppState;
use crate::model::time_series::TimeSeriesPoint;
use crate::model::test::TestUpdate;

/// WebSocket handler that upgrades the connection and forwards to the handle_socket function
pub async fn handle_ws(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("WebSocket connection request received");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle the WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(tokio::sync::Mutex::new(sender));
    let (tx, mut rx) = mpsc::channel::<Message>(2048);

    // Try to set this as the active connection
    if !state.set_ws_connection(tx.clone()).await {
        // Another connection exists, close this one
        if let Err(e) = sender.lock().await.send(Message::Close(None)).await {
            warn!("Failed to send close message: {}", e);
        }
        return;
    }

    info!("WebSocket connection established");

    // Send initial time series data
    let time_series_points = state.get_time_series_points().await;
    if !time_series_points.is_empty() {
        let msg = json!({
            "type": "time_series_history",
            "data": time_series_points
        });
        if let Ok(json) = serde_json::to_string(&msg) {
            if let Err(e) = sender.lock().await.send(Message::Text(json)).await {
                warn!("Failed to send initial time series data: {}", e);
                state.remove_ws_connection().await;
                return;
            }
        }
    }

    let sender_clone = Arc::clone(&sender);
    let mut last_ping_response = std::time::Instant::now();
    let ping_timeout = std::time::Duration::from_secs(90); // 90 second timeout

    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => {
                    info!("Received close message from client");
                    break;
                },
                Message::Ping(data) => {
                    if let Err(e) = sender_clone.lock().await.send(Message::Pong(data)).await {
                        warn!("Failed to send pong: {}", e);
                        break;
                    }
                },
                Message::Pong(_) => {
                    last_ping_response = std::time::Instant::now();
                },
                _ => continue,
            }

            // Check if we haven't received a ping response in too long
            if last_ping_response.elapsed() > ping_timeout {
                warn!("WebSocket ping timeout - no response in {:?}", ping_timeout);
                break;
            }
        }
        info!("WebSocket receive loop ended");
    });

    // Handle outgoing messages
    let mut send_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            tokio::select! {
                Some(msg) = rx.recv() => {
                    match sender.lock().await.send(msg).await {
                        Ok(_) => {},
                        Err(e) => {
                            warn!("Failed to send WebSocket message: {}", e);
                            break;
                        }
                    }
                }
                _ = interval.tick() => {
                    // Send ping to keep connection alive
                    if let Err(e) = sender.lock().await.send(Message::Ping(vec![])).await {
                        warn!("Failed to send ping: {}", e);
                        break;
                    }
                }
                else => break
            }
        }
        info!("WebSocket send loop ended");
    });

    // Wait for either task to complete
    tokio::select! {
        _ = (&mut recv_task) => {
            send_task.abort();
            info!("WebSocket receive task completed");
        },
        _ = (&mut send_task) => {
            recv_task.abort();
            info!("WebSocket send task completed");
        },
    }

    // Clean up
    state.remove_ws_connection().await;
    info!("WebSocket connection closed and cleaned up");
}

// Send a test update to all connected WebSocket clients
pub fn send_test_update(clients: &[Sender<Message>], update: TestUpdate) {
    let msg = json!({
        "type": "test_update",
        "data": update
    });

    if let Ok(json) = serde_json::to_string(&msg) {
        for client in clients {
            match client.try_send(Message::Text(json.clone())) {
                Ok(_) => {},
                Err(e) => match e {
                    mpsc::error::TrySendError::Full(_) => {
                        warn!("Client message queue is full, dropping message");
                    },
                    mpsc::error::TrySendError::Closed(_) => {
                        warn!("Client is disconnected, message not sent");
                    }
                }
            }
        }
    }
}

// Send a time series update to all connected WebSocket clients
pub fn send_time_series_update(clients: &[Sender<Message>], point: TimeSeriesPoint) {
    let msg = json!({
        "type": "time_series_update",
        "data": point
    });

    if let Ok(json) = serde_json::to_string(&msg) {
        for client in clients {
            match client.try_send(Message::Text(json.clone())) {
                Ok(_) => {},
                Err(e) => match e {
                    mpsc::error::TrySendError::Full(_) => {
                        warn!("Client message queue is full");
                    },
                    mpsc::error::TrySendError::Closed(_) => {
                        warn!("Client is disconnected");
                    }
                }
            }
        }
    }
}