// /frontend/js/ws.js
import { state } from './state.js';
import { log } from './ui.js';

const API_URL = `${window.location.origin}/api`;
let webSocket = null;
let heartbeatInterval = null;

// Главная функция для установки соединения
export function connectWebSocket() {
    // Если соединение уже есть или в процессе, ничего не делаем
    if (webSocket && webSocket.readyState < 2) { // 0=CONNECTING, 1=OPEN
        return;
    }
    
    // Убеждаемся, что у нас есть токен для аутентификации
    if (!state.token) {
        log("WebSocket connection failed: No auth token.");
        return;
    }

    const wsUrl = API_URL.replace(/^http/, 'ws') + `/ws?token=${state.token}`;
    
    log("Connecting to WebSocket...");
    webSocket = new WebSocket(wsUrl);

    // --- НАЧАЛО ОБНОВЛЕНИЙ ---

    webSocket.onopen = () => {
        log("WebSocket connection established. Real-time log active.");

        // Убиваем старый интервал, если он вдруг остался
        if (heartbeatInterval) clearInterval(heartbeatInterval);

        // Запускаем новый "heartbeat" каждые 30 секунд
        heartbeatInterval = setInterval(() => {
            if (webSocket && webSocket.readyState === WebSocket.OPEN) {
                // Отправляем специальное ping-сообщение.
                // Сервер должен быть готов его получить и ответить pong'ом.
                webSocket.send('__ping__');
            }
        }, 30000); // 30 секунд
    };

    webSocket.onmessage = (event) => {
        // Сервер будет отвечать на наш ping сообщением '__pong__'.
        // Мы должны игнорировать эти служебные сообщения.
        if (event.data === '__pong__') {
            // Можно добавить логику для проверки, что "сердцебиение" работает,
            // но пока просто игнорируем.
            return; 
        }

        try {
            const data = JSON.parse(event.data);
            handleIncomingMessage(data);
        } catch (e) {
            log(`WS << Received non-JSON message: ${event.data}`);
        }
    };

    webSocket.onclose = () => {
        log("WebSocket connection closed. Attempting to reconnect in 5 seconds...");
        
        // Обязательно останавливаем интервал, чтобы не было утечек
        if (heartbeatInterval) clearInterval(heartbeatInterval);
        heartbeatInterval = null;

        webSocket = null;
        setTimeout(connectWebSocket, 5000);
    };

    webSocket.onerror = (error) => {
        log("WebSocket error occurred.");
        console.error("WebSocket Error:", error);
        // onclose будет вызван автоматически после ошибки, так что он обработает реконнект.
    };
    
    // --- КОНЕЦ ОБНОВЛЕНИЙ ---
}

// Обработка входящих сообщений от сервера (без изменений)
function handleIncomingMessage(data) {
    log(`WS << [${data.type}] Received`);
    
    switch (data.type) {
        case 'user_joined':
            log(`Operator "${data.username}" connected to your location.`);
            break;
        case 'user_left':
            log(`Operator "${data.username}" disconnected from your location.`);
            break;
        default:
            log(`Unknown WS message type: ${data.type}`);
    }
}