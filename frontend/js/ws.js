// /frontend/js/ws.js
import { log, updatePresenceList, dom } from './ui.js';
import { state } from './state.js';

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
    
    // Получаем текущий список пользователей из UI (самый надежный способ)
    // Мы читаем DOM, чтобы получить актуальное состояние
    const currentUsersOnScreen = Array.from(dom.presenceList.querySelectorAll('li'))
                                       .map(li => li.innerText.replace('> ', ''))
                                       .filter(name => !name.startsWith('[')); // Убираем "[ SCANNING... ]"
    
    let updatedUsers = [...currentUsersOnScreen]; // Создаем копию

    switch (data.type) {
        case 'room_state':
            // Просто заменяем текущий список на тот, что пришел от сервера
            updatedUsers = data.users || [];
            break;
        case 'user_joined':
            // Добавляем нового, если его еще нет
            if (!presentUsers.some(u => u.id === data.user.id)) {
                presentUsers.push(data.user);
            }
            log(`Operator "${data.user.username}" connected.`);
            break;
        case 'user_left':
            // Сервер прислал ID и имя ушедшего
            presentUsers = presentUsers.filter(u => u.id !== data.user_id);
            log(`Operator "${data.username}" disconnected.`);
            break;
        default:
            log(`Unknown WS message type: ${data.type}`);
            return;
    }
    
    // Вызываем обновление UI с финальным, обновленным списком
    updatePresenceList(updatedUsers);
}