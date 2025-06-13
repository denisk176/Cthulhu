use crate::mqtt::MQTTBroadcast;
use crate::web::WebState;
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::response::Response;

pub async fn serial_handler(
    State(state): State<WebState>,
    Path(port_label): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(|ws| serial_handle_socket(ws, state, port_label))
}

async fn serial_handle_socket(mut socket: WebSocket, state: WebState, port_label: String) {
    let mut receiver = state.broadcast.subscribe();

    let port = state.manager.get_port(&port_label).await.unwrap();
    socket
        .send(Message::Binary(Bytes::from(port.log_buffer)))
        .await
        .unwrap();

    while let Ok(message) = receiver.recv().await {
        match message {
            MQTTBroadcast::SerialData { label, data } => {
                if label == port_label {
                    socket
                        .send(Message::Binary(Bytes::from(data)))
                        .await
                        .unwrap();
                }
            }
            _ => {}
        }
    }
}
