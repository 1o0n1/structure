// /var/www/structure/server/src/ws/mod.rs

// Объявляем наши под-модули
pub mod handler;
pub mod state;
pub mod utils;

// Публично экспортируем только то, что нужно снаружи:
// - Главный обработчик для роутера.
// - Функцию для перемещения для player_handler.
// - Тип состояния для AppState.
pub use handler::ws_handler;
pub use state::WsState;
pub use utils::change_room;