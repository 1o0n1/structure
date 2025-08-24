// /server/src/handlers/player_handler.rs
use crate::{auth::Claims, error::AppError, models::player::Player, state::AppState};
use axum::{extract::State, Extension, Json};

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