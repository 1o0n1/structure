// /var/www/structure/server/src/main.rs
use axum::Router;
use dotenvy::dotenv;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Подключаем все наши модули
mod auth;
mod config;
mod db;
mod error;
mod handlers;
mod models;
mod routes;
mod state;

use config::Config;
use state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    let pool: PgPool = db::connect_db(&config.database_url).await;

    let app_state = AppState { pool, config };

    let cors = CorsLayer::new().allow_origin(Any).allow_headers(vec![
        axum::http::header::AUTHORIZATION,
        axum::http::header::CONTENT_TYPE,
    ]).allow_methods(tower_http::cors::Any);

    let app = Router::new()
        .nest("/api", routes::create_router(app_state.clone()))
        // Раздаем статику из папки frontend, а не static
        .nest_service("/", ServeDir::new("frontend"))
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3003));
    tracing::debug!("->> СЕРВЕР ЗАПУЩЕН на http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}