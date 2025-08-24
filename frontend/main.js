// /frontend/main.js

// 1. Импортируем крипто-модуль
import init from './pkg/crypto.js';

// 2. Импортируем наши собственные модули
import { dom, log } from './js/ui.js';
import { checkAuthState, handleRegister, handleLogin, handleLogout } from './js/auth.js';

// 3. Главная функция - точка входа
async function main() {
    // Инициализируем WASM
    await init();
    log("WASM Crypto Core Loaded.");

    // Вешаем обработчики событий на кнопки
    dom.registerBtn.addEventListener('click', handleRegister);
    dom.loginBtn.addEventListener('click', handleLogin);
    dom.logoutBtn.addEventListener('click', handleLogout);

    // Проверяем, залогинен ли пользователь
    checkAuthState();
}

// 4. Запускаем все
main();