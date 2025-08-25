// /frontend/js/ui.js
import { handleAction } from './auth.js';
// Находим все элементы один раз
export const dom = {
    authView: document.getElementById('auth-view'),
    terminalView: document.getElementById('terminal-view'),
    emailInput: document.getElementById('email'),
    usernameInput: document.getElementById('username'),
    passwordInput: document.getElementById('password'),
    registerBtn: document.getElementById('register-btn'),
    loginBtn: document.getElementById('login-btn'),
    logoutBtn: document.getElementById('logout-btn'),
    playerUsername: document.getElementById('player-username'),
    playerRole: document.getElementById('player-role'),
    playerAccessLevel: document.getElementById('player-access-level'),
    locationImage: document.getElementById('location-image'),
    locationName: document.getElementById('location-name'),
    locationDescription: document.getElementById('location-description'),
    actionList: document.getElementById('action-list'),
    logContent: document.getElementById('log-content'),
    presenceList: document.getElementById('presence-list'),
};

export function log(message) {
    const logEntry = document.createElement('p');
    logEntry.innerHTML = `> ${message}`;
    dom.logContent.appendChild(logEntry);
    dom.logContent.scrollTop = dom.logContent.scrollHeight;
}

export function showAuthView() {
    dom.authView.style.display = 'flex';
    dom.terminalView.style.display = 'none';
}

export function showTerminalView() {
    dom.authView.style.display = 'none';
    dom.terminalView.style.display = 'grid';
}

export function updatePlayerStatus(player, claims) {
    dom.playerUsername.innerText = claims?.username || 'UNKNOWN';
    dom.playerRole.innerText = claims?.role || 'UNKNOWN';
    dom.playerAccessLevel.innerText = player.access_level;
}

export function updateLocationUI(response) {
    const { location, links } = response;
    dom.locationName.innerText = location.name;
    dom.locationDescription.innerText = location.description;
    if (location.image_url) {
        dom.locationImage.src = location.image_url;
        dom.locationImage.style.display = 'block';
    } else {
        dom.locationImage.style.display = 'none';
    }
    dom.actionList.innerHTML = ''; // Очищаем старые действия
    if (links && links.length > 0) {
        links.forEach(link => {
            const li = document.createElement('li');
            li.innerText = link.link_text;
            li.dataset.targetId = link.target_location_id; // Сохраняем ID цели
            li.addEventListener('click', () => handleAction(link.target_location_id));
            dom.actionList.appendChild(li);
        });
    } else {
        dom.actionList.innerHTML = '<li>[ NO ACTIONS AVAILABLE ]</li>';
    }

    log(`Successfully loaded location: ${location.name}`);
}

export function updatePresenceList(users) {
    console.log('UI_LOG: updatePresenceList called with users:', users);
    dom.presenceList.innerHTML = ''; // Очищаем список
    if (users && users.length > 0) {
        users.forEach(username => {
            const li = document.createElement('li');
            li.innerText = `> ${username}`;
            dom.presenceList.appendChild(li);
        });
    } else {
        dom.presenceList.innerHTML = '<li>[ NO OTHER SIGNALS DETECTED ]</li>';
    }
}

