use crate::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use tokio::sync::broadcast;

pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| client_loop(socket, state.events.clone()))
}

async fn client_loop(mut socket: WebSocket, tx: broadcast::Sender<serde_json::Value>) {
    let mut rx = tx.subscribe();
    tracing::info!("WebSocket client connected");

    loop {
        tokio::select! {
            evt = rx.recv() => {
                match evt {
                    Ok(json) => {
                        let text = serde_json::to_string(&json).unwrap_or_default();
                        if socket.send(Message::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("WS client lagged, dropped {} events", n);
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(_)) => break,
                    _ => {}
                }
            }
        }
    }
    tracing::info!("WebSocket client disconnected");
}
