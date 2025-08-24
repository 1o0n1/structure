// /frontend/js/auth.js
import { api } from './api.js';
import { dom, log, showAuthView, showTerminalView, updatePlayerStatus, updateLocationUI } from './ui.js';
import { state, saveSession, loadSession, clearSession } from './state.js';
import { generate_keypair_base64, encrypt_with_password, decrypt_with_password } from '../pkg/crypto.js';

export function checkAuthState() {
    loadSession();
    if (state.token) {
        log("Active session found. Initializing terminal...");
        initializeTerminal();
    } else {
        log("No active session found. Please log in.");
        showAuthView();
    }
}

export async function handleRegister() {
    const { email, username, password } = getAuthInput();
    if (!email || !username || !password) return log("Error: All fields are required.");
    
    log("Generating cryptographic keys...");
    const [secretKeyB64, publicKeyB64] = generate_keypair_base64();
    
    try {
        log("Encrypting private key with your password...");
        const encryptedPrivateKey = encrypt_with_password(password, secretKeyB64);
        
        const result = await api.register({ email, username, password, public_key: publicKeyB64, encrypted_private_key: encryptedPrivateKey });
        log(`Registration successful for ${result.username}. Please log in.`);
    } catch (error) {
        log(`Error: ${error.message}`);
    }
}

export async function handleLogin() {
    const { email, password } = getAuthInput();
    if (!email || !password) return log("Error: Email and password are required.");

    log("Sending login request...");
    try {
        const result = await api.login({ email, password });
        log("Login successful! Decrypting session keys...");

        state.token = result.token;

        if (result.encrypted_private_key) {
            state.privateKey = decrypt_with_password(password, result.encrypted_private_key);
            log("Private key decrypted.");
        }
        
        saveSession();
        loadSession(); // Перезагружаем состояние, чтобы распарсить claims
        initializeTerminal();
    } catch (error) {
        log(`Error: ${error.message}`);
    }
}

export function handleLogout() {
    log("Disconnecting from STRUCTURE...");
    clearSession();
    showAuthView();
}

async function initializeTerminal() {
    showTerminalView();
    try {
        const player = await api.getPlayerStatus();
        updatePlayerStatus(player, state.claims);
        
        if (player.current_location_id) {
            const location = await api.getLocation(player.current_location_id, player.access_level);
            updateLocationUI(location);
        }
    } catch (error) {
        log(`Session Error: ${error.message}`);
        handleLogout();
    }
}

function getAuthInput() {
    return {
        email: dom.emailInput.value,
        username: dom.usernameInput.value,
        password: dom.passwordInput.value,
    };
}