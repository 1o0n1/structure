// /var/www/structure/server/src/routes/mod.rs

use crate::{
    auth::auth_middleware,
    handlers::{location_handler, player_handler, user_handler},
    state::AppState,
    ws::handler::ws_handler,
};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

/// Создает роутер для всех API-эндпоинтов (/register, /login, /player/status и т.д.).
/// Этот роутер затем будет вложен под префикс /api в main.rs.
pub fn create_router(app_state: AppState) -> Router {
    // Публичные роуты, доступные всем
    let public_routes = Router::new()
        .route("/register", post(user_handler::register))
        .route("/login", post(user_handler::login));

    // Защищенные роуты, требующие валидного JWT-токена
    let protected_routes = Router::new()
        .route("/player/status", get(player_handler::get_player_status))
        .route("/player/move", post(player_handler::move_player))
        .route("/locations/:id", get(location_handler::get_location))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    // Собираем все роуты вместе и применяем к ним общее состояние.
    // БЕЗ .nest("/api", ...) - это делается в main.rs
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .route("/ws", get(ws_handler))
        .with_state(app_state)
}