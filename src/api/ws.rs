use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade},
    response::Response,
};
use axum::extract::ws::{Message, WebSocket};
use tracing::{debug, warn};

use crate::state::AppState;

/// GET /api/v1/system/stream
///
/// Upgrades the connection to a WebSocket and streams all broadcast game
/// events to the spectator.  The bootstrap sequence (subscribe → snapshot →
/// replay buffered) is the client's responsibility per websocket_protocol.md §5.
pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.ws_tx.subscribe();

    loop {
        tokio::select! {
            // Incoming broadcast event → forward to client
            result = rx.recv() => {
                match result {
                    Ok(msg) => {
                        if socket.send(Message::Text((*msg).clone().into())).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!("WebSocket subscriber lagged by {n} messages");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            // Incoming client frame (close / ping handled by Axum)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }

    debug!("WebSocket connection closed");
}
