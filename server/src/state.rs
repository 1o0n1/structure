// /var/www/structure/server/src/state.rs
use crate::config::Config;
use crate::ws::state::WsState;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub ws_state: WsState,
}