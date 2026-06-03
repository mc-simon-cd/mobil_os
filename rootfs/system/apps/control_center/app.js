// Copyright 2026 mcsimon
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Control Center Dashboard Logic (app.js)

const host = window.location.hostname || 'localhost';
// Make API requests to the gateway on port 8085 (if testing on host) or 8080 (guest default)
// Detect if running on host (e.g. if page is loaded from localhost:8000 or similar)
const apiPort = (window.location.port === '8000' || window.location.port === '8085') ? '8085' : '8080';
const API_BASE = `http://${host}:${apiPort}/api`;

let currentLang = 'en';
let translations = {};

// Translation mapping for UI static labels that are not in locale files directly
const uiLabels = {
    en: {
        'txt-power-mgmt': 'Power Management',
        'txt-battery-label': 'Battery',
        'txt-current-mode-label': 'Current Mode:',
        'txt-mode-powersave': 'Power Saver',
        'txt-mode-balanced': 'Balanced',
        'txt-mode-performance': 'Performance',
        'txt-input-sim': 'Input Injection Simulator',
        'txt-last-event': 'Last Event Info',
        'txt-evt-type': 'Type:',
        'txt-evt-code': 'Code:',
        'txt-evt-value': 'Value:',
        'txt-quick-actions': 'Quick Controls',
        'btn-action-power': 'Inject Power',
        'btn-action-volup': 'Inject Vol +',
        'btn-action-voldown': 'Inject Vol -',
        'btn-action-home': 'Inject Home',
        'txt-terminal-log': 'System Log Console',
    },
    tr: {
        'txt-power-mgmt': 'Güç Yönetimi',
        'txt-battery-label': 'Batarya',
        'txt-current-mode-label': 'Mevcut Mod:',
        'txt-mode-powersave': 'Güç Tasarrufu',
        'txt-mode-balanced': 'Dengeli',
        'txt-mode-performance': 'Performans',
        'txt-input-sim': 'Giriş Enjeksiyon Simülatörü',
        'txt-last-event': 'Son Olay Bilgisi',
        'txt-evt-type': 'Tip:',
        'txt-evt-code': 'Kod:',
        'txt-evt-value': 'Değer:',
        'txt-quick-actions': 'Hızlı Kontroller',
        'btn-action-power': 'Güç Enjekte Et',
        'btn-action-volup': 'Ses + Enjekte Et',
        'btn-action-voldown': 'Ses - Enjekte Et',
        'btn-action-home': 'Ana Ekran Enjekte Et',
        'txt-terminal-log': 'Sistem Günlüğü Konsolu',
    },
    es: {
        'txt-power-mgmt': 'Gestión de Energía',
        'txt-battery-label': 'Batería',
        'txt-current-mode-label': 'Modo Actual:',
        'txt-mode-powersave': 'Ahorro de Energía',
        'txt-mode-balanced': 'Equilibrado',
        'txt-mode-performance': 'Rendimiento',
        'txt-input-sim': 'Simulador de Inyección de Entrada',
        'txt-last-event': 'Info de Último Evento',
        'txt-evt-type': 'Tipo:',
        'txt-evt-code': 'Código:',
        'txt-evt-value': 'Valor:',
        'txt-quick-actions': 'Controles Rápidos',
        'btn-action-power': 'Inyectar Encendido',
        'btn-action-volup': 'Inyectar Vol +',
        'btn-action-voldown': 'Inyectar Vol -',
        'btn-action-home': 'Inyectar Inicio',
        'txt-terminal-log': 'Consola de Registro del Sistema',
    },
    fr: {
        'txt-power-mgmt': 'Gestion de l\'Énergie',
        'txt-battery-label': 'Batterie',
        'txt-current-mode-label': 'Mode Actuel:',
        'txt-mode-powersave': 'Économiseur',
        'txt-mode-balanced': 'Équilibré',
        'txt-mode-performance': 'Performance',
        'txt-input-sim': 'Simulateur d\'Injection d\'Entrée',
        'txt-last-event': 'Infos Dernier Événement',
        'txt-evt-type': 'Type:',
        'txt-evt-code': 'Code:',
        'txt-evt-value': 'Valeur:',
        'txt-quick-actions': 'Controles Rapides',
        'btn-action-power': 'Injecter Alim',
        'btn-action-volup': 'Injecter Vol +',
        'btn-action-voldown': 'Injecter Vol -',
        'btn-action-home': 'Injecter Accueil',
        'txt-terminal-log': 'Console de Journal Système',
    },
    de: {
        'txt-power-mgmt': 'Energieverwaltung',
        'txt-battery-label': 'Batterie',
        'txt-current-mode-label': 'Aktueller Modus:',
        'txt-mode-powersave': 'Energiesparmodus',
        'txt-mode-balanced': 'Ausgewogen',
        'txt-mode-performance': 'Leistung',
        'txt-input-sim': 'Eingabeinjektions-Simulator',
        'txt-last-event': 'Letzte Ereignisinfo',
        'txt-evt-type': 'Typ:',
        'txt-evt-code': 'Code:',
        'txt-evt-value': 'Wert:',
        'txt-quick-actions': 'Schnellsteuerung',
        'btn-action-power': 'Power injizieren',
        'btn-action-volup': 'Vol + injizieren',
        'btn-action-voldown': 'Vol - injizieren',
        'btn-action-home': 'Home injizieren',
        'txt-terminal-log': 'Systemprotokollkonsole',
    },
    pt: {
        'txt-power-mgmt': 'Gerenciamento de Energia',
        'txt-battery-label': 'Bateria',
        'txt-current-mode-label': 'Modo Atual:',
        'txt-mode-powersave': 'Economia de Energia',
        'txt-mode-balanced': 'Equilibrado',
        'txt-mode-performance': 'Desempenho',
        'txt-input-sim': 'Simulador de Injeção de Entrada',
        'txt-last-event': 'Info do Último Evento',
        'txt-evt-type': 'Tipo:',
        'txt-evt-code': 'Código:',
        'txt-evt-value': 'Valor:',
        'txt-quick-actions': 'Controles Rápidos',
        'btn-action-power': 'Injetar Energia',
        'btn-action-volup': 'Injetar Vol +',
        'btn-action-voldown': 'Injetar Vol -',
        'btn-action-home': 'Injetar Início',
        'txt-terminal-log': 'Console de Registro do Sistema',
    },
    ru: {
        'txt-power-mgmt': 'Управление питанием',
        'txt-battery-label': 'Батарея',
        'txt-current-mode-label': 'Текущий режим:',
        'txt-mode-powersave': 'Энергосбережение',
        'txt-mode-balanced': 'Сбалансированный',
        'txt-mode-performance': 'Производительность',
        'txt-input-sim': 'Симулятор ввода',
        'txt-last-event': 'Последнее событие',
        'txt-evt-type': 'Тип:',
        'txt-evt-code': 'Код:',
        'txt-evt-value': 'Значение:',
        'txt-quick-actions': 'Быстрые действия',
        'btn-action-power': 'Питание',
        'btn-action-volup': 'Громкость +',
        'btn-action-voldown': 'Громкость -',
        'btn-action-home': 'Домой',
        'txt-terminal-log': 'Консоль системного журнала',
    },
    zh: {
        'txt-power-mgmt': '电源管理',
        'txt-battery-label': '电池',
        'txt-current-mode-label': '当前模式:',
        'txt-mode-powersave': '省电模式',
        'txt-mode-balanced': '均衡模式',
        'txt-mode-performance': '性能模式',
        'txt-input-sim': '输入注入模拟器',
        'txt-last-event': '最近事件信息',
        'txt-evt-type': '类型:',
        'txt-evt-code': '代码:',
        'txt-evt-value': '数值:',
        'txt-quick-actions': '快捷控制',
        'btn-action-power': '注入电源键',
        'btn-action-volup': '注入音量加',
        'btn-action-voldown': '注入音量减',
        'btn-action-home': '注入主页键',
        'txt-terminal-log': '系统日志控制台',
    },
    ja: {
        'txt-power-mgmt': '電源管理',
        'txt-battery-label': 'バッテリー',
        'txt-current-mode-label': '現在のモード:',
        'txt-mode-powersave': '省電力',
        'txt-mode-balanced': 'バランス',
        'txt-mode-performance': 'パフォーマンス',
        'txt-input-sim': '入力インジェクションシミュレータ',
        'txt-last-event': '最後のイベント情報',
        'txt-evt-type': 'タイプ:',
        'txt-evt-code': 'コード:',
        'txt-evt-value': '値:',
        'txt-quick-actions': 'クイックコントロール',
        'btn-action-power': '電源キー注入',
        'btn-action-volup': '音量＋注入',
        'btn-action-voldown': '音量－注入',
        'btn-action-home': 'ホームキー注入',
        'txt-terminal-log': 'システムログコンソール',
    },
    ar: {
        'txt-power-mgmt': 'إدارة الطاقة',
        'txt-battery-label': 'البطارية',
        'txt-current-mode-label': 'الوضع الحالي:',
        'txt-mode-powersave': 'موفر الطاقة',
        'txt-mode-balanced': 'متوازن',
        'txt-mode-performance': 'الأداء العالي',
        'txt-input-sim': 'محاكي حقن الإدخال',
        'txt-last-event': 'معلومات الحدث الأخير',
        'txt-evt-type': 'النوع:',
        'txt-evt-code': 'الكود:',
        'txt-evt-value': 'القيمة:',
        'txt-quick-actions': 'التحكم السريع',
        'btn-action-power': 'حقن زر التشغيل',
        'btn-action-volup': 'حقن رفع الصوت',
        'btn-action-voldown': 'حقن خفض الصوت',
        'btn-action-home': 'حقن زر الرئيسية',
        'txt-terminal-log': '콘솔 لوحة تحكم النظام',
    }
};

// Console Log Helper
const consoleLogContent = document.getElementById('console-log-content');
function addLog(message, level = 'info') {
    const entry = document.createElement('div');
    entry.className = `log-entry log-${level}`;
    const timestamp = new Date().toLocaleTimeString();
    entry.innerText = `[${timestamp}] ${message}`;
    consoleLogContent.appendChild(entry);
    consoleLogContent.scrollTop = consoleLogContent.scrollHeight;
}

// Clear Console
document.getElementById('btn-clear-console').addEventListener('click', () => {
    consoleLogContent.innerHTML = '';
    addLog('Console log cleared.');
});

// Load Translation Files from locale/ directory
async function loadTranslations(langCode) {
    try {
        const response = await fetch(`./locale/${langCode}.txt`);
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        const text = await response.text();
        const parsed = {};
        text.split('\n').forEach(line => {
            line = line.trim();
            if (!line || line.startsWith('#')) return;
            const sepIdx = line.indexOf('=');
            if (sepIdx !== -1) {
                const key = line.substring(0, sepIdx).trim();
                const val = line.substring(sepIdx + 1).trim();
                parsed[key] = val;
            }
        });
        translations = parsed;
        addLog(`Loaded locale translations for: ${langCode}`, 'success');
        applyLocalization(langCode);
    } catch (e) {
        addLog(`Failed to fetch locale file from './locale/${langCode}.txt'. Using inline fallback translations.`, 'warning');
        applyLocalization(langCode);
    }
}

// Apply localization to UI elements
function applyLocalization(langCode) {
    currentLang = langCode;
    
    // UI Elements
    const elementsToTranslate = uiLabels[langCode] || uiLabels['en'];
    for (const [id, val] of Object.entries(elementsToTranslate)) {
        const el = document.getElementById(id);
        if (el) {
            // Keep children like icons intact if any
            const textSpan = el.querySelector('span:not([id])') || el;
            if (textSpan === el) {
                el.innerText = val;
            } else {
                textSpan.innerText = val;
            }
        }
    }
    
    // Header dropdown display
    const currentLangLabel = document.getElementById('current-lang-label');
    const langNames = {
        en: 'English', tr: 'Türkçe', es: 'Español', fr: 'Français',
        de: 'Deutsch', pt: 'Português', ru: 'Русский', zh: '中文',
        ja: '日本語', ar: 'العربية'
    };
    currentLangLabel.innerText = langNames[langCode] || langCode;

    // Refresh dynamic labels on screen
    updateBatteryDisplay();
}

// Language Selector UI Interactivity
const langSelectBtn = document.getElementById('lang-select-btn');
const langDropdownOptions = document.getElementById('lang-dropdown-options');

langSelectBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    langDropdownOptions.classList.toggle('show');
});

document.addEventListener('click', () => {
    langDropdownOptions.classList.remove('show');
});

document.querySelectorAll('.lang-opt').forEach(opt => {
    opt.addEventListener('click', () => {
        const selected = opt.getAttribute('data-lang');
        loadTranslations(selected);
    });
});

// Battery Circle Progress Bar Animation
const batteryProgressBar = document.getElementById('battery-progress-bar');
const batteryLevelVal = document.getElementById('battery-level-val');
const mockBatteryIndicator = document.getElementById('mock-battery-indicator');

let lastBatteryLevel = 0;
let lastPowerMode = 'balanced';

function setBatteryLevel(pct) {
    lastBatteryLevel = pct;
    batteryLevelVal.innerText = `${pct}%`;
    mockBatteryIndicator.innerText = `${pct}%`;

    const radius = 70;
    const circumference = 2 * Math.PI * radius; // 439.82
    const offset = circumference - (pct / 100) * circumference;
    batteryProgressBar.style.strokeDashoffset = offset;

    // Dynamically change battery progress color based on mode/level
    const stop1 = document.querySelector('#battery-grad stop:nth-child(1)');
    const stop2 = document.querySelector('#battery-grad stop:nth-child(2)');
    
    if (pct < 20) {
        // Red critical battery
        stop1.setAttribute('stop-color', '#ef4444');
        stop2.setAttribute('stop-color', '#b91c1c');
    } else if (lastPowerMode === 'powersave') {
        // Green energy saving battery
        stop1.setAttribute('stop-color', '#10b981');
        stop2.setAttribute('stop-color', '#059669');
    } else if (lastPowerMode === 'performance') {
        // Orange performance power battery
        stop1.setAttribute('stop-color', '#f59e0b');
        stop2.setAttribute('stop-color', '#d97706');
    } else {
        // Balanced Blue battery
        stop1.setAttribute('stop-color', '#3b82f6');
        stop2.setAttribute('stop-color', '#1d4ed8');
    }
}

function updateBatteryDisplay() {
    setBatteryLevel(lastBatteryLevel);
    
    // Map raw power mode string to translation key
    const currentPowerModeVal = document.getElementById('current-power-mode-val');
    const localizedModeKey = `power.mode.${lastPowerMode}`;
    const modeString = translations[localizedModeKey] || 
                       (uiLabels[currentLang] && uiLabels[currentLang][`txt-mode-${lastPowerMode}`]) || 
                       lastPowerMode;
                       
    currentPowerModeVal.innerText = modeString;
    
    // Style the current mode indicator container
    currentPowerModeVal.className = 'mode-value';
    currentPowerModeVal.style.color = '#ffffff';
    if (lastPowerMode === 'performance') {
        currentPowerModeVal.style.background = 'rgba(245, 158, 11, 0.2)';
        currentPowerModeVal.style.borderColor = 'rgba(245, 158, 11, 0.4)';
    } else if (lastPowerMode === 'powersave') {
        currentPowerModeVal.style.background = 'rgba(16, 185, 129, 0.2)';
        currentPowerModeVal.style.borderColor = 'rgba(16, 185, 129, 0.4)';
    } else {
        currentPowerModeVal.style.background = 'rgba(59, 130, 246, 0.2)';
        currentPowerModeVal.style.borderColor = 'rgba(59, 130, 246, 0.4)';
    }
}

// Fetch System Status (API Polling)
async function fetchSystemStatus() {
    try {
        const response = await fetch(`${API_BASE}/power`);
        if (!response.ok) {
            throw new Error(`API Gateway returned HTTP ${response.status}`);
        }
        const data = await response.json();
        
        const isModeChanged = (lastPowerMode !== data.power_mode);
        lastPowerMode = data.power_mode;
        
        setBatteryLevel(data.battery_level || 0);
        updateBatteryDisplay();

        // Update active class on buttons
        document.querySelectorAll('.mode-btn').forEach(btn => {
            btn.classList.remove('active');
        });
        const currentBtn = document.getElementById(`btn-mode-${data.power_mode}`);
        if (currentBtn) {
            currentBtn.classList.add('active');
        }

        if (isModeChanged) {
            addLog(`Power mode synchronized with system: ${data.power_mode.toUpperCase()}`, 'info');
        }
    } catch (e) {
        addLog(`System status connection error: ${e.message}`, 'error');
    }
}

// Fetch Last Input Event
async function fetchLastInputEvent() {
    try {
        const response = await fetch(`${API_BASE}/input/last`);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }
        const data = await response.json();
        if (data.status === 'success') {
            document.getElementById('evt-type-val').innerText = data.type;
            document.getElementById('evt-code-val').innerText = data.code;
            document.getElementById('evt-value-val').innerText = data.value;
            
            // Map standard linux codes to text label
            let keyname = 'UNKNOWN';
            if (data.type === 1) { // EV_KEY
                if (data.code === 116) keyname = 'KEY_POWER';
                else if (data.code === 115) keyname = 'KEY_VOLUMEUP';
                else if (data.code === 114) keyname = 'KEY_VOLUMEDOWN';
                else if (data.code === 102) keyname = 'KEY_HOME';
            }
            document.getElementById('mock-screen-text').innerText = `${keyname} (${data.value === 1 ? 'DOWN' : 'UP'})`;
        }
    } catch (e) {
        // Silent catch for polling last event
    }
}

// Change Power Mode (API Trigger)
async function setPowerMode(mode) {
    addLog(`Requesting power mode change to: ${mode.toUpperCase()}...`, 'info');
    try {
        const response = await fetch(`${API_BASE}/power/mode?mode=${mode}`, {
            method: 'POST'
        });
        if (!response.ok) {
            throw new Error(`HTTP Error ${response.status}`);
        }
        const data = await response.json();
        if (data.status === 'success') {
            addLog(`Power mode successfully changed to: ${mode.toUpperCase()}`, 'success');
            lastPowerMode = mode;
            updateBatteryDisplay();
            // Refetch to sync quickly
            fetchSystemStatus();
        } else {
            addLog(`Failed to change power mode: ${data.message || 'Unknown response'}`, 'error');
        }
    } catch (e) {
        addLog(`Error setting power mode: ${e.message}`, 'error');
    }
}

// Bind Mode Buttons
document.querySelectorAll('.mode-btn').forEach(btn => {
    btn.addEventListener('click', () => {
        const mode = btn.getAttribute('data-mode');
        setPowerMode(mode);
    });
});

// Inject Input Key Event (API Trigger)
async function injectKeyEvent(type, code, value) {
    try {
        const response = await fetch(`${API_BASE}/input/inject?type=${type}&code=${code}&value=${value}`, {
            method: 'POST'
        });
        if (!response.ok) {
            throw new Error(`HTTP Error ${response.status}`);
        }
        const data = await response.json();
        return data.status === 'success';
    } catch (e) {
        addLog(`Injection failed: ${e.message}`, 'error');
        return false;
    }
}

// Click Trigger Helper: down then up
async function simulateKeyClick(code, keyName) {
    addLog(`Simulating click on ${keyName} (Code: ${code})...`, 'info');
    
    // 1. Key DOWN
    const downSuccess = await injectKeyEvent(1, code, 1);
    if (downSuccess) {
        addLog(`Injected ${keyName} DOWN (value=1)`, 'success');
        // Fetch to display instantly
        setTimeout(fetchLastInputEvent, 50);
        
        // 2. Key UP after 100ms
        setTimeout(async () => {
            const upSuccess = await injectKeyEvent(1, code, 0);
            if (upSuccess) {
                addLog(`Injected ${keyName} UP (value=0)`, 'success');
                setTimeout(fetchLastInputEvent, 50);
            }
        }, 100);
    } else {
        addLog(`Failed to inject ${keyName} event.`, 'error');
    }
}

// Bind Action Buttons
document.getElementById('btn-action-power').addEventListener('click', () => simulateKeyClick(116, 'KEY_POWER'));
document.getElementById('btn-action-volup').addEventListener('click', () => simulateKeyClick(115, 'KEY_VOLUMEUP'));
document.getElementById('btn-action-voldown').addEventListener('click', () => simulateKeyClick(114, 'KEY_VOLUMEDOWN'));
document.getElementById('btn-action-home').addEventListener('click', () => simulateKeyClick(102, 'KEY_HOME'));

// Bind Mock Device Hardware Buttons
document.getElementById('btn-hw-power').addEventListener('click', () => simulateKeyClick(116, 'KEY_POWER'));
document.getElementById('btn-hw-vol-up').addEventListener('click', () => simulateKeyClick(115, 'KEY_VOLUMEUP'));
document.getElementById('btn-hw-vol-down').addEventListener('click', () => simulateKeyClick(114, 'KEY_VOLUMEDOWN'));

// Bind Mock Screen Navigation Bar Buttons (Back, Home, Recent)
document.getElementById('sim-home').addEventListener('click', () => simulateKeyClick(102, 'KEY_HOME'));
document.getElementById('sim-back').addEventListener('click', () => simulateKeyClick(158, 'KEY_BACK')); // 158 = KEY_BACK
document.getElementById('sim-recent').addEventListener('click', () => simulateKeyClick(139, 'KEY_MENU')); // 139 = KEY_MENU

// Polling intervals
setInterval(fetchSystemStatus, 2000);
setInterval(fetchLastInputEvent, 2000);

// Initialize Page
addLog(`Connecting to API Gateway at ${API_BASE}...`);
loadTranslations('en');
fetchSystemStatus();
fetchLastInputEvent();
