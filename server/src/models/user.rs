use serde::{Serialize, Deserialize};
use sqlx::Type;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, Type, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "PascalCase")]
pub enum UserRole {
    User,
    Moderator,
    Architect,
    Admin,
    Creator,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub public_key: Option<String>,
    #[serde(skip_serializing)]
    pub encrypted_private_key: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct PublicUser {
    pub id: Uuid,
    pub username: String,
    pub public_key: Option<String>,
    pub role: UserRole,
}