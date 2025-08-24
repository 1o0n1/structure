// /var/www/structure/server/src/db.rs
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn connect_db(database_url: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .expect("Не удалось подключиться к базе данных")
}