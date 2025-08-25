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
    // Игнорируем query, так как берем access_level из БД
    Query(_query): Query<AccessQuery>, 
) -> Result<Json<LocationResponse>, AppError> {
    tracing::debug!("Запрос локации {} для пользователя {}", id, claims.sub);

    // --- ИСПРАВЛЕНИЕ: ПЕРЕИМЕНОВЫВАЕМ ПЕРЕМЕННЫЕ ---
    let location_info = sqlx::query_as!(
        Location,
        "SELECT * FROM locations WHERE id = $1",
        id
    ).fetch_one(&state.pool).await?;

    let links_info = sqlx::query_as!(
        LocationLink,
        "SELECT target_location_id, link_text, required_access_level FROM location_links WHERE source_location_id = $1",
        id
    ).fetch_all(&state.pool).await?;
    
    let player = sqlx::query_as!(crate::models::player::Player, "SELECT * FROM players WHERE user_id = $1", claims.sub)
        .fetch_one(&state.pool).await?;

    if player.access_level < location_info.security_level {
        let scrambled_location = Location {
            name: "[[ДАННЫЕ ПОВРЕЖДЕНЫ]]".to_string(),
            description: format!("ТРЕБУЕТСЯ УРОВЕНЬ ДОСТУПА: {}. ВАШ УРОВЕНЬ: {}.", location_info.security_level, player.access_level),
            image_url: Some("/static/images/scrambled.gif".to_string()),
            ..location_info
        };
        let scrambled_response = LocationResponse { location: scrambled_location, links: vec![] };
        return Ok(Json(scrambled_response));
    }
    
    // Используем правильные имена переменных
    Ok(Json(LocationResponse { location: location_info, links: links_info }))
}