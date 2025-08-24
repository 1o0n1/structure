use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Player {
    pub user_id: Uuid,
    pub current_location_id: Option<Uuid>,
    pub access_level: i32,
    pub inventory: Option<Value>,
}