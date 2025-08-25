use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, // <-- ИЗМЕНЕНИЕ: Используем Query для извлечения параметров
        State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;
use crate::{auth::Claims, error::AppError, state::AppState}; // <-- ИЗМЕНЕНИЕ: Используем единый AppState
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize; // <-- НОВЫЙ ИМПОРТ

// НОВАЯ СТРУКТУРА для парсинга токена из URL
#[derive(Debug, Deserialize)]
pub struct AuthQuery {
    token: String,
}

#[derive(Debug, Clone)]
pub struct WsState {
    pub tx: broadcast::Sender<(crate::models::message::Message, Uuid)>,
    pub online_users: Arc<Mutex<HashMap<Uuid, String>>>,
}

impl WsState {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { tx, online_users: Arc::new(Mutex::new(HashMap::new())) }
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>, // <-- ИЗМЕНЕНИЕ: Получаем единый AppState
    Query(query): Query<AuthQuery>, // <-- ИЗМЕНЕНИЕ: Элегантно получаем токен
) -> Result<Response, AppError> {
    let claims = decode::<Claims>(
        &query.token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?
    .claims;

    let user_id = claims.sub;
    // ИЗМЕНЕНИЕ: Клонируем WsState из общего состояния
    let ws_state = state.ws_state.clone(); 

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, user_id, ws_state)))
}


async fn handle_socket(socket: WebSocket, user_id: Uuid, ws_state: WsState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = ws_state.tx.subscribe();

    ws_state.online_users.lock().await.insert(user_id, "online".to_string());
    tracing::debug!("User {} connected via WebSocket.", user_id);

    let mut send_task = tokio::spawn(async move {
        while let Ok((msg, sender_id)) = rx.recv().await {
            if msg.recipient_id == Some(user_id) || sender_id == user_id {
                let msg_json = serde_json::to_string(&msg).unwrap_or_default();
                // ИЗМЕНЕНИЕ: Конвертируем String в тип, который ожидает Axum
                if sender.send(Message::Text(msg_json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg { break; }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    ws_state.online_users.lock().await.remove(&user_id);
    tracing::debug!("User {} disconnected.", user_id);
}