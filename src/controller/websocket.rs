use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use std::sync::Arc;
use std::time::Duration;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::time::interval;

use crate::model::state::AppState;

/// WebSocket handler that upgrades the connection and forwards to the handle_socket function
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::info!("WebSocket connection request received");
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle the WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    tracing::info!("WebSocket connection established");
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to broadcast channel
    let mut rx = state.tx.subscribe();
    
    // Create heartbeat interval
    let mut heartbeat_interval = interval(Duration::from_secs(30)); // 30-second heartbeat
    
    // Spawn task to forward broadcast messages to WebSocket
    let mut send_task = tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle messages from the broadcast channel
                result = rx.recv() => {
                    match result {
                        Ok(msg) => {
                            if sender.send(Message::Text(msg)).await.is_err() {
                                break;
                            }
                        },
                        Err(e) => {
                            tracing::error!("Broadcast channel error: {}", e);
                            break;
                        }
                    }
                },
                // Send periodic heartbeat
                _ = heartbeat_interval.tick() => {
                    if let Ok(ping) = serde_json::to_string(&json!({"type": "ping", "timestamp": chrono::Utc::now().timestamp()})) {
                        if sender.send(Message::Text(ping)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    });
    
    // Handle incoming WebSocket messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    tracing::debug!("Received WebSocket message: {}", text);
                    
                    // Process commands from client
                    if text == "ping" {
                        if let Ok(pong) = serde_json::to_string(&json!({"type": "pong", "timestamp": chrono::Utc::now().timestamp()})) {
                            if let Err(e) = state.tx.send(pong) {
                                tracing::error!("Failed to send pong: {}", e);
                            }
                        }
                    }
                }
                Message::Ping(data) => {
                    // Automatically respond to pings
                    if sender.send(Message::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Message::Close(frame) => {
                    tracing::info!("WebSocket close frame received: {:?}", frame);
                    break;
                }
                _ => {}
            }
        }
        
        tracing::info!("WebSocket receiver task completed");
    });
    
    // Wait for either task to complete
    tokio::select! {
        _ = (&mut send_task) => {
            tracing::info!("WebSocket sender task completed, aborting receiver");
            recv_task.abort();
        },
        _ = (&mut recv_task) => {
            tracing::info!("WebSocket receiver task completed, aborting sender");
            send_task.abort();
        },
    }
    
    tracing::info!("WebSocket connection closed");
} 