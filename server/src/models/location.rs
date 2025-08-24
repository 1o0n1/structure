use serde::Serialize;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Location {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub image_url: Option<String>,
    pub security_level: i32,
    pub creator_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}