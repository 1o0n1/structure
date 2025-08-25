// /var/www/structure/server/src/ws/utils.rs

use super::state::Room;
use crate::{state::AppState, auth::Claims};
use axum::extract::ws::Message;
use uuid::Uuid;

/// Отправляет сообщение всем клиентам в комнате, кроме одного (skip_user)
pub fn broadcast_message(room: &Room, message: String, skip_user: Uuid) {
    for (user_id, (_, tx)) in room.iter() {
        if *user_id != skip_user {
            let _ = tx.send(Message::Text(message.clone()));
        }
    }
}

/// Перемещает клиента из старой комнаты в новую и рассылает уведомления.
pub async fn change_room(
    state: &AppState,
    claims: &Claims,
    old_room_id: Option<Uuid>,
    new_room_id: Uuid,
) {
    let mut rooms = state.ws_state.rooms.lock().await;

    // 1. Забираем клиента из старой комнаты и оповещаем о выходе
    let client_tuple = if let Some(old_id) = old_room_id {
        if let Some(room) = rooms.get_mut(&old_id) {
            let client_data = room.remove(&claims.sub);
            let leave_msg = format!(r#"{{"type": "user_left", "username": "{}"}}"#, &claims.username);
            broadcast_message(room, leave_msg, Uuid::nil());
            client_data
        } else {
            None
        }
    } else {
        None
    };

    // 2. Если мы успешно забрали клиента, вставляем его в новую комнату
    if let Some(client_data) = client_tuple {
        let room = rooms.entry(new_room_id).or_default();
        
        let current_users: Vec<String> = room.values().map(|(name, _)| name.clone()).collect();
        let room_state_msg = serde_json::to_string(&serde_json::json!({
            "type": "room_state",
            "users": current_users,
        })).unwrap_or_default();
        let _ = client_data.1.send(Message::Text(room_state_msg));

        let join_msg = format!(r#"{{"type": "user_joined", "username": "{}"}}"#, &claims.username);
        broadcast_message(room, join_msg, Uuid::nil());
        room.insert(claims.sub, client_data);
    } else {
        tracing::warn!("Не удалось переместить WS-клиента {}, так как он не найден в старой комнате.", &claims.username);
    }
}