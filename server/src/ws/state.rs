// /var/www/structure/server/src/ws/state.rs

use axum::extract::ws::Message;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use uuid::Uuid;

// Клиент: его имя и канал для отправки сообщений
pub type Client = (String, tokio::sync::mpsc::UnboundedSender<Message>);
// Комната: user_id -> Client
pub type Room = HashMap<Uuid, Client>;

// Состояние WebSocket: location_id -> Room
#[derive(Clone)]
pub struct WsState {
    pub rooms: Arc<Mutex<HashMap<Uuid, Room>>>,
}

impl WsState {
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}