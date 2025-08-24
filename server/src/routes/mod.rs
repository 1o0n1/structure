// /server/src/routes/mod.rs
use crate::{auth::auth_middleware, handlers::{user_handler, player_handler, location_handler}, state::AppState};
use axum::{middleware, routing::{get, post}, Router}; // Возвращаем get и middleware

pub fn create_router(app_state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/register", post(user_handler::register))
        .route("/login", post(user_handler::login));

    // --- РАСКОММЕНТИРУЕМ И НАСТРАИВАЕМ ---
    let protected_routes = Router::new()
        .route("/player/status", get(player_handler::get_player_status))
        .route("/locations/:id", get(location_handler::get_location))
        .layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes) // <-- ОБЪЕДИНЯЕМ
        .with_state(app_state)
}