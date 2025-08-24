// /server/src/handlers/location_handler.rs
use crate::{auth::Claims, error::AppError, models::location::Location, state::AppState};
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AccessQuery {
    access_level: Option<i32>,
}

pub async fn get_location(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>, // Этот роут защищен
    Path(id): Path<Uuid>,
    Query(query): Query<AccessQuery>,
) -> Result<Json<Location>, AppError> {
    tracing::debug!("Запрос локации {} для пользователя {}", id, claims.sub);

    let location = sqlx::query_as!(
        Location,
        "SELECT * FROM locations WHERE id = $1",
        id
    )
    .fetch_one(&state.pool)
    .await?;

    let player_access_level = query.access_level.unwrap_or(0);

    // TODO: Реализовать логику "зашифрованной" локации, если player_access_level < location.security_level
    // Пока что просто отдаем локацию как есть.

    Ok(Json(location))
}