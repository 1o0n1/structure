use crate::{
    auth::Claims,
    error::AppError,
    models::{user::{User, UserRole}}, // Убираем неиспользуемый Player
    state::AppState,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateUserPayload {
    pub username: String,
    pub email: String,
    pub password: String,
    pub public_key: String,
    pub encrypted_private_key: String,
}

#[derive(Serialize)]
pub struct SafeUser {
    pub id: Uuid,
    pub username: String,
    pub role: UserRole,
    pub public_key: Option<String>,
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserPayload>,
) -> Result<Json<SafeUser>, AppError> {
    let password = payload.password.clone();
     let password_hash =
        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            // Сразу превращаем результат в строку, чтобы разорвать ссылку
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map(|hash| hash.to_string()) // <--- ВОТ КЛЮЧЕВОЕ ИЗМЕНЕНИЕ
        })
        .await.map_err(|_| AppError::InternalServerError)? // Ошибка, если задача паникует
        .map_err(AppError::PasswordHashError)?; // Ошибка, если `hash_password` не удался

    let mut tx = state.pool.begin().await?;

    let new_user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (username, email, password_hash, public_key, encrypted_private_key)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, username, email, role AS "role: _", public_key, encrypted_private_key, password_hash, created_at, updated_at
        "#,
        payload.username, payload.email, password_hash.to_string(),
        payload.public_key, payload.encrypted_private_key
    ).fetch_one(&mut *tx).await?;

    let start_location_id = Uuid::parse_str("a1b2c3d4-e5f6-7890-1234-567890abcdef").unwrap();
    sqlx::query!(
        "INSERT INTO players (user_id, current_location_id) VALUES ($1, $2)",
        new_user.id,
        start_location_id
    ).execute(&mut *tx).await?;

    tx.commit().await?;

    Ok(Json(SafeUser {
        id: new_user.id,
        username: new_user.username,
        role: new_user.role,
        public_key: new_user.public_key,
    }))
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    encrypted_private_key: Option<String>,
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = sqlx::query_as!(
        User,
        r#"SELECT id, username, email, role as "role: _", public_key, encrypted_private_key, password_hash, created_at, updated_at
         FROM users WHERE email = $1"#,
        payload.email
    )
    .fetch_optional(&state.pool).await?.ok_or(AppError::InvalidCredentials)?;

    let password = payload.password.clone();
    let password_hash_str = user.password_hash.clone();

    let is_valid = tokio::task::spawn_blocking(move || {
        let parsed_hash = argon2::PasswordHash::new(&password_hash_str)?;
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash)
    }).await.map_err(|_| AppError::InternalServerError)?;

    if is_valid.is_err() {
        return Err(AppError::InvalidCredentials);
    }

    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        pk: user.public_key.clone().unwrap_or_default(),
        role: user.role,
        exp: (Utc::now() + Duration::days(7)).timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.config.jwt_secret.as_ref()),
    )?;

    Ok(Json(AuthResponse {
        token,
        encrypted_private_key: user.encrypted_private_key,
    }))
}