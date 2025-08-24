// /server/src/handlers/player_handler.rs
use crate::{auth::Claims, error::AppError, models::player::Player, state::AppState};
use axum::{extract::State, Extension, Json};
use serde::Deserialize;
use uuid::Uuid;

// Этот хендлер будет доступен только аутентифицированным пользователям
pub async fn get_player_status(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>, // Middleware вставит сюда claims из токена
) -> Result<Json<Player>, AppError> {
    let player = sqlx::query_as!(
        Player,
        "SELECT * FROM players WHERE user_id = $1",
        claims.sub // claims.sub - это ID пользователя из токена
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(player))
}

#[derive(Deserialize)]
pub struct MovePayload {
    pub target_location_id: Uuid,
}

pub async fn move_player(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(payload): Json<MovePayload>,
) -> Result<(), AppError> {
    // TODO: Проверить, что переход из текущей локации в целевую вообще возможен!

    sqlx::query!(
        "UPDATE players SET current_location_id = $1 WHERE user_id = $2",
        payload.target_location_id,
        claims.sub
    )
    .execute(&state.pool)
    .await?;

    Ok(()) // Возвращаем пустой успешный ответ
}