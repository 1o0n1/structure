// /frontend/js/state.js

// Это не настоящее глобальное состояние, а просто объект-хранилище.
// Каждый модуль будет импортировать его и сможет читать/писать.
export const state = {
    token: null,
    privateKey: null,
    claims: null, // Распарсенный JWT-токен
};

// --- Функции для работы с localStorage ---

export function saveSession() {
    if (state.token) localStorage.setItem('jwtToken', state.token);
    if (state.privateKey) localStorage.setItem('userPrivateKey', state.privateKey);
}

export function loadSession() {
    state.token = localStorage.getItem('jwtToken');
    state.privateKey = localStorage.getItem('userPrivateKey');
    if (state.token) {
        state.claims = getClaimsFromToken(state.token);
    }
}

export function clearSession() {
    state.token = null;
    state.privateKey = null;
    state.claims = null;
    localStorage.removeItem('jwtToken');
    localStorage.removeItem('userPrivateKey');
}

function getClaimsFromToken(token) {
    if (!token) return null;
    try {
        return JSON.parse(atob(token.split('.')[1]));
    } catch (e) {
        console.error("Failed to parse token", e);
        return null;
    }
}