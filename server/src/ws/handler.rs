// /var/www/structure/server/src/ws/handler.rs
use super::utils::broadcast_message;
use crate::{auth::Claims, error::AppError, state::AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures::{stream::StreamExt, SinkExt};
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct WsQuery {
    pub token: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let claims = decode::<Claims>(
        &query.token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &Validation::default(),
    )?.claims;
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, claims)))
}

async fn handle_socket(socket: WebSocket, state: AppState, claims: Claims) {
    let user_id = claims.sub;
    let username = claims.username.clone();
    tracing::info!("WebSocket client connected: {} ({})", &username, user_id);

    let (mut sender, mut receiver) = socket.split();
    let (tx,mut rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let player_location_id = sqlx::query_scalar!(
        "SELECT current_location_id FROM players WHERE user_id = $1", user_id
    ).fetch_one(&state.pool).await.unwrap_or(None).unwrap_or_else(Uuid::nil);

    {
        let mut rooms = state.ws_state.rooms.lock().await;
        let room = rooms.entry(player_location_id).or_default();

        let current_users: Vec<String> = room.values().map(|(name, _)| name.clone()).collect();
        let room_state_msg = serde_json::to_string(&serde_json::json!({
            "type": "room_state", "users": current_users,
        })).unwrap_or_default();
        let _ = tx.send(Message::Text(room_state_msg));

        let join_msg = format!(r#"{{"type": "user_joined", "username": "{}"}}"#, &username);
        broadcast_message(room, join_msg, user_id);

        room.insert(user_id, (username.clone(), tx.clone()));
    }

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) if text == "__ping__" => {
                let _ = tx.send(Message::Text("__pong__".to_string()));
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    {
        let mut rooms = state.ws_state.rooms.lock().await;
        if let Some(room) = rooms.get_mut(&player_location_id) {
            room.remove(&user_id);
            let leave_msg = format!(r#"{{"type": "user_left", "username": "{}"}}"#, &username);
            broadcast_message(room, leave_msg, user_id);
        }
    }
    tracing::info!("WebSocket client disconnected: {}", &username);
}