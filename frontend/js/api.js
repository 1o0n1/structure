// /frontend/js/api.js
import { state } from './state.js';
import { log } from './ui.js';

const API_URL = `${window.location.origin}/api`;

// Вспомогательная функция для всех fetch-запросов
async function apiFetch(endpoint, options = {}) {
    const headers = {
        'Content-Type': 'application/json',
        ...options.headers,
    };

    if (state.token) {
        headers['Authorization'] = `Bearer ${state.token}`;
    }

    const response = await fetch(`${API_URL}${endpoint}`, {
        ...options,
        headers,
    });

    if (!response.ok) {
        const errorData = await response.json().catch(() => ({ error: 'Unknown server error' }));
        throw new Error(errorData.error || `HTTP error! status: ${response.status}`);
    }

    // Если тело ответа пустое (например, для DELETE), возвращаем null
    if (response.status === 204) {
        return null;
    }

    return response.json();
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
};