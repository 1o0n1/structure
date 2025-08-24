// /frontend/js/api.js
import { state } from './state.js';
import { log } from './ui.js';

const API_URL = `${window.location.origin}/api`;

// Вспомогательная функция для всех fetch-запросов
async function apiFetch(endpoint, options = {}) {
    const headers = { 'Content-Type': 'application/json', ...options.headers, };
    if (state.token) { headers['Authorization'] = `Bearer ${state.token}`; }

    const response = await fetch(`${API_URL}${endpoint}`, { ...options, headers, });

    if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: response.statusText }));
        throw new Error(errorData.error || `HTTP error! status: ${response.status}`);
    }

    // --- УМНОЕ РЕШЕНИЕ ---
    // Пробуем прочитать тело как текст. Если оно пустое, мы не упадем с ошибкой.
    const text = await response.text();
    // Если текст не пустой, пытаемся его распарсить как JSON.
    // Если пустой, просто возвращаем null.
    return text ? JSON.parse(text) : null;
}

// Функции для конкретных эндпоинтов
export const api = {
    register: (data) => apiFetch('/register', {
        method: 'POST',
        body: JSON.stringify(data),
    }),
    login: (data) => apiFetch('/login', {
        method: 'POST',
        body: JSON.stringify(data),
    }),
    getPlayerStatus: () => apiFetch('/player/status'),
    getLocation: (id, accessLevel) => apiFetch(`/locations/${id}?access_level=${accessLevel}`),
    movePlayer: (targetLocationId) => apiFetch('/player/move', {
        method: 'POST',
        body: JSON.stringify({ target_location_id: targetLocationId }),
    }),
};