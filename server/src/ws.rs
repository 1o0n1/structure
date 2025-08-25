// /var/www/structure/server/src/ws.rs

use crate::{auth::Claims, error::AppError, state::AppState};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

// --- Состояние WebSocket ---
type ClientTx = tokio::sync::mpsc::UnboundedSender<Message>;
type Room = HashMap<Uuid, ClientTx>; // Ключ - user_id
#[derive(Clone)]
pub struct WsState {
    pub rooms: Arc<Mutex<HashMap<Uuid, Room>>>, // Ключ - location_id
}
impl WsState {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// --- Обработчик WebSocket-запроса ---

#[derive(serde::Deserialize)]
pub struct WsQuery {
    pub token: String,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Аутентифицируем пользователя по токену из URL
    let claims = decode::<Claims>(
        &query.token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &Validation::default(),
    )?
    .claims;

    // Если токен валиден, "апгрейдим" HTTP-соединение до WebSocket
    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, claims)))
}


/// Главная функция, управляющая одним WebSocket-соединением
async fn handle_socket(socket: WebSocket, state: AppState, claims: Claims) {
    let user_id = claims.sub;
    tracing::info!("WebSocket client connected: {} ({})", claims.username, user_id);

    // Разделяем сокет на отправителя и получателя
    let (mut sender, mut receiver) = socket.split();

    // Создаем MPSC-канал (почтовый ящик) для этого клиента
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // Эта задача (task) будет вечно слушать "почтовый ящик" (rx)
    // и пересылать любые сообщения в реальный сокет (sender).
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                // Если отправить не удалось, значит клиент отключился
                break;
            }
        }
    });

    // --- Логика при ПЕРВОМ подключении ---
    let player_location_id: Uuid =
        sqlx::query_scalar("SELECT current_location_id FROM players WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&state.pool)
            .await
            .unwrap_or(None) // Если запрос к БД упал, считаем что ID нет
            .unwrap_or_else(Uuid::nil); // Если ID в БД NULL, используем Uuid::nil

    // Добавляем клиента в его текущую комнату
    {
        let mut rooms = state.ws_state.rooms.lock().await;
        let room = rooms.entry(player_location_id).or_default();
        let join_msg = format!(r#"{{"type": "user_joined", "username": "{}"}}"#, claims.username);
        broadcast_message(room, join_msg, user_id);
        room.insert(user_id, tx.clone());
    } // Блокировка `rooms` снимается здесь

    // --- Главный цикл обработки сообщений ОТ клиента ---
    loop {
        tokio::select! {
            // Получаем сообщение от клиента (из реального сокета)
            Some(Ok(msg)) = receiver.next() => {
                match msg {
                    Message::Text(text) => {
                        // Проверяем, не heartbeat ли это
                        if text == "__ping__" {
                            // Отвечаем pong'ом, чтобы клиент знал, что мы живы
                            if tx.send(Message::Text("__pong__".to_string())).is_err() {
                                // Если отправить не удалось, клиент отвалился, выходим
                                break;
                            }
                        } else {
                            // TODO: Здесь будет обработка других команд от клиента (чат и т.д.)
                            tracing::info!("WS << Received text from {}: {}", claims.username, text);
                        }
                    }
                    Message::Close(_) => {
                        // Клиент инициировал закрытие
                        break;
                    }
                    _ => { /* Игнорируем бинарные и другие типы сообщений */ }
                }
            }
            // Канал tx был закрыт, значит, другая часть программы хочет нас закрыть
            else => {
                break;
            }
        }
    }

    // --- Логика при ОТКЛЮЧЕНИИ клиента ---
    {
        let mut rooms = state.ws_state.rooms.lock().await;
        if let Some(room) = rooms.get_mut(&player_location_id) {
            room.remove(&user_id);
            let leave_msg = format!(r#"{{"type": "user_left", "username": "{}"}}"#, claims.username);
            broadcast_message(room, leave_msg, user_id);
        }
    }
    tracing::info!("WebSocket client disconnected: {}", claims.username);
}

// --- Утилиты для рассылки ---

/// Отправляет сообщение всем клиентам в комнате, кроме одного (skip_user)
fn broadcast_message(room: &Room, message: String, skip_user: Uuid) {
    for (user_id, tx) in room.iter() {
        if *user_id != skip_user {
            // `send` неблокирующий, поэтому нам не нужен `await`
            // `let _ = ...` подавляет предупреждение о неиспользуемом `Result`
            let _ = tx.send(Message::Text(message.clone()));
        }
    }
}

/// Перемещает клиента из старой комнаты в новую и рассылает уведомления.
/// Эта функция будет вызываться из `player_handler`
pub async fn change_room(
    state: &AppState,
    user_id: Uuid,
    username: &str,
    old_room_id: Option<Uuid>,
    new_room_id: Uuid,
) {
    let mut rooms = state.ws_state.rooms.lock().await;

    // 1. Забираем канал клиента из старой комнаты и оповещаем о выходе
    let client_tx = if let Some(old_id) = old_room_id {
        if let Some(room) = rooms.get_mut(&old_id) {
            let tx = room.remove(&user_id); // Забираем канал клиента
            let leave_msg = format!(r#"{{"type": "user_left", "username": "{}"}}"#, username);
            broadcast_message(room, leave_msg, Uuid::nil()); // Оповещаем всех
            tx // Возвращаем канал
        } else {
            None
        }
    } else {
        None
    };

    // 2. Если мы успешно забрали канал, вставляем клиента в новую комнату
    if let Some(tx) = client_tx {
        let room = rooms.entry(new_room_id).or_default();
        let join_msg = format!(r#"{{"type": "user_joined", "username": "{}"}}"#, username);
        broadcast_message(room, join_msg, Uuid::nil()); // Оповещаем всех
        room.insert(user_id, tx); // Вставляем клиента в новую комнату
    } else {
        // Это может произойти, если игрок переместился, будучи оффлайн,
        // или если его WebSocket-соединение еще не установлено. Это не ошибка.
        tracing::warn!("Не удалось переместить WS-клиента {}, так как он не найден в старой комнате.", username);
    }
}