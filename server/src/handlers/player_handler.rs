// /server/src/handlers/player_handler.rs
use crate::{auth::Claims, error::AppError, models::{player::Player, location::Location}, state::AppState};
use axum::{extract::State, Extension, Json, http::StatusCode};
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
) ->  Result<StatusCode, AppError> {
    // 1. Получаем текущее состояние игрока
    let player = sqlx::query_as!(Player, "SELECT * FROM players WHERE user_id = $1", claims.sub)
        .fetch_one(&state.pool).await?;

    // 2. Получаем данные о целевой локации
    let target_location = sqlx::query_as!(Location, "SELECT * FROM locations WHERE id = $1", payload.target_location_id)
        .fetch_one(&state.pool).await?;
    
    // 3. Проверяем, достаточно ли у игрока прав доступа
    if player.access_level < target_location.security_level {
        tracing::warn!("Попытка несанкционированного доступа от {} к локации {}", claims.sub, target_location.id);
        // Возвращаем ошибку. На фронте это попадет в catch.
        return Err(AppError::Unauthorized); 
    }

    let old_location_id = player.current_location_id;

    sqlx::query!(
        "UPDATE players SET current_location_id = $1 WHERE user_id = $2",
        payload.target_location_id,
        claims.sub
    ).execute(&state.pool).await?;

  //      if let Some(old_id) = old_location_id {
  //      ws::broadcast_leave(&state.ws_state, old_id, &claims.username).await;
  //      }
  //      ws::broadcast_join(&state.ws_state, payload.target_location_id, &claims.username).await;

    Ok(StatusCode::NO_CONTENT)
}