// /frontend/main.js

import init from './pkg/crypto.js';
import { dom } from './js/ui.js';
import { checkAuthState, handleRegister, handleLogin, handleLogout, handleAction } from './js/auth.js';

// Главная функция - точка входа
async function main() {
    await init();
    dom.logContent.innerHTML = ''; // Очищаем лог при старте
    log("WASM Crypto Core Loaded.");

    // Вешаем обработчики на кнопки аутентификации
    dom.registerBtn.addEventListener('click', handleRegister);
    dom.loginBtn.addEventListener('click', handleLogin);
    dom.logoutBtn.addEventListener('click', handleLogout);

    // --- РАЗРЫВАЕМ ЦИКЛ: ИСПОЛЬЗУЕМ ДЕЛЕГИРОВАНИЕ СОБЫТИЙ ---
    // Вешаем один обработчик на весь список действий.
    // Это эффективнее и решает проблему циклической зависимости.
    dom.actionList.addEventListener('click', (event) => {
        const targetLi = event.target.closest('li[data-target-id]');
        if (targetLi) {
            handleAction(targetLi.dataset.targetId);
        }
    });

    // Проверяем, залогинен ли пользователь
    checkAuthState();
}

// Утилита для логгирования, нужна до загрузки ui.js
function log(message) {
    const logContainer = document.getElementById('log-content');
    if (!logContainer) { console.log(message); return; }
    const logEntry = document.createElement('p');
    logEntry.innerHTML = `> ${message}`;
    logContainer.appendChild(logEntry);
    logContainer.scrollTop = logContainer.scrollHeight;
}


main();