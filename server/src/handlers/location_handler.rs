// /server/src/handlers/location_handler.rs
use crate::{
    auth::Claims,
    error::AppError,
    // Добавляем LocationLink
    models::{location::Location, link::LocationLink},
    state::AppState,
};

use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;



#[derive(Deserialize)]
pub struct AccessQuery {
    access_level: Option<i32>,
}

#[derive(Serialize)]
pub struct LocationResponse {
    location: Location,
    links: Vec<LocationLink>,
}

pub async fn get_location(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<Uuid>,
    Query(_query): Query<AccessQuery>,
) -> Result<Json<LocationResponse>, AppError> {
    tracing::debug!("Запрос локации {} для пользователя {}", id, claims.sub);

    // Получаем данные о самой локации
    let location = sqlx::query_as!(
        Location,
        "SELECT * FROM locations WHERE id = $1",
        id
    ).fetch_one(&state.pool).await?;

    // Получаем доступные переходы из этой локации
    let links = sqlx::query_as!(
        LocationLink,
        "SELECT target_location_id, link_text, required_access_level FROM location_links WHERE source_location_id = $1",
        id
    ).fetch_all(&state.pool).await?;
    
    // TODO: Проверять access_level и security_level

    Ok(Json(LocationResponse { location, links }))
}