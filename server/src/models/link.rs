// /server/src/models/link.rs
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct LocationLink {
    pub target_location_id: Uuid,
    pub link_text: String,
    pub required_access_level: i32,
}