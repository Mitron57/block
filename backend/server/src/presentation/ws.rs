use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use futures::StreamExt;
use serde::Deserialize;
use uuid::Uuid;

use crate::presentation::state::AppState;
use crate::ws_protocol::{parse_client_ws_message, ClientWsMessage, ServerWsMessage};

#[derive(Deserialize)]
pub struct WsAuth {
    pub token: String,
}

pub async fn board_ws(
    ws: WebSocketUpgrade,
    Query(auth): Query<WsAuth>,
    State(state): State<AppState>,
    Path(board_id): Path<Uuid>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state, board_id, auth.token))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, board_id: Uuid, token: String) {
    let user_id = match state.auth.verify_token(&token) {
        Ok(u) => u,
        Err(_) => {
            let _ = socket
                .send(Message::Text(
                    serde_json::to_string(&ServerWsMessage::Error {
                        message: "unauthorized".into(),
                    })
                    .unwrap_or_else(|_| r#"{"op":"error","message":"unauthorized"}"#.into())
                    .into(),
                ))
                .await;
            return;
        }
    };

    if state.boards.get_board(board_id, user_id).await.is_err() {
        let _ = socket
            .send(Message::Text(
                serde_json::to_string(&ServerWsMessage::Error {
                    message: "forbidden".into(),
                })
                .unwrap_or_default()
                .into(),
            ))
            .await;
        return;
    }

    let elements = match state.boards.list_elements(board_id, user_id).await {
        Ok(e) => e,
        Err(_) => return,
    };
    let snap = serde_json::to_string(&ServerWsMessage::Snapshot { elements }).unwrap_or_default();
    if socket.send(Message::Text(snap.into())).await.is_err() {
        return;
    }

    let mut rx = state.rooms.subscribe(board_id).await;

    loop {
        tokio::select! {
            inc = socket.next() => {
                let Some(msg) = inc else { break; };
                let Ok(msg) = msg else { break; };
                if let Message::Text(t) = msg {
                    let text = t.as_str();
                    let parsed = match parse_client_ws_message(text) {
                        Ok(p) => p,
                        Err(_) => {
                            let _ = socket.send(Message::Text(serde_json::to_string(&ServerWsMessage::Error {
                                message: "invalid json".into(),
                            }).unwrap_or_default().into())).await;
                            continue;
                        }
                    };
                    match apply_ws_message(&state, board_id, user_id, parsed).await {
                        Ok(Some(outgoing)) => {
                            state.rooms.publish(board_id, outgoing).await;
                        }
                        Ok(None) => {}
                        Err(e) => {
                            let _ = socket.send(Message::Text(serde_json::to_string(&ServerWsMessage::Error {
                                message: e,
                            }).unwrap_or_default().into())).await;
                        }
                    }
                } else if matches!(msg, Message::Close(_)) {
                    break;
                }
            }
            room = rx.recv() => {
                let Ok(text) = room else { continue; };
                if socket.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
        }
    }
}

async fn apply_ws_message(
    state: &AppState,
    board_id: Uuid,
    user_id: Uuid,
    msg: ClientWsMessage,
) -> Result<Option<String>, String> {
    match msg {
        ClientWsMessage::AddElement {
            element_type,
            payload,
            z_index,
        } => {
            let el = state
                .boards
                .add_element(board_id, user_id, &element_type, payload, z_index)
                .await
                .map_err(|e| e.to_string())?;
            Ok(Some(
                serde_json::to_string(&ServerWsMessage::ElementAdded { element: el })
                    .map_err(|e| e.to_string())?,
            ))
        }
        ClientWsMessage::RemoveElement { id } => {
            state
                .boards
                .remove_element(board_id, user_id, id)
                .await
                .map_err(|e| e.to_string())?;
            Ok(Some(
                serde_json::to_string(&ServerWsMessage::ElementRemoved { id })
                    .map_err(|e| e.to_string())?,
            ))
        }
        ClientWsMessage::Clear => {
            state
                .boards
                .clear_elements(board_id, user_id)
                .await
                .map_err(|e| e.to_string())?;
            Ok(Some(
                serde_json::to_string(&ServerWsMessage::Cleared).map_err(|e| e.to_string())?,
            ))
        }
    }
}
