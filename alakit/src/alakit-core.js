// Alakit Core JavaScript Bridge
// Ez a fájl felelős az IPC kommunikációért és az automatikus eseménykezelésért.

// 1. Core Alap híd
window.alakit = function (msg) {
    if (window.ipc && window.ipc.postMessage) {
        window.ipc.postMessage(msg);
    } else {
        console.error("Alakit Error: IPC híd nem található!");
    }
};

// --- ÚJ: Biztonság - Kontextus menü tiltása ---
document.addEventListener('contextmenu', event => event.preventDefault());
// --------------------------------------------

// --- ÚJ: Konzol Átirányítás és Hibakezelés ---
// Minden console.log, warn, error átirányítása a Rust terminálba
const originalConsole = {
    log: console.log,
    warn: console.warn,
    error: console.error
};

function sendLogToRust(level, args) {
    const message = Array.from(args).map(arg =>
        typeof arg === 'object' ? JSON.stringify(arg) : String(arg)
    ).join(' ');
    window.alakit(`alakit:log|{"level":"${level}","msg":"${message}"}`);
}

console.log = function () {
    sendLogToRust("info", arguments);
    originalConsole.log.apply(console, arguments);
};
console.warn = function () {
    sendLogToRust("warn", arguments);
    originalConsole.warn.apply(console, arguments);
};
console.error = function () {
    sendLogToRust("error", arguments);
    originalConsole.error.apply(console, arguments);
};

// Globális JS hibák elkapása
window.onerror = function (message, source, lineno, colno, error) {
    const errorMsg = `JS Error: ${message} at ${source}:${lineno}:${colno}`;
    window.alakit(`alakit:log|{"level":"error","msg":"${errorMsg}"}`);
    return false;
};
// --------------------------------------------

// Segédfüggvény a Form/beviteli adatok összegyűjtésére
function gatherFormData(formId) {
    const form = document.getElementById(formId);
    if (!form) return "";

    const data = {};
    const elements = form.querySelectorAll('input, select, textarea');
    elements.forEach(el => {
        if (!el.name && !el.id) return; // Kell identifier
        const key = el.name || el.id;

        if (el.type === 'checkbox') {
            data[key] = el.checked;
        } else if (el.type === 'radio') {
            if (el.checked) data[key] = el.value;
        } else {
            data[key] = el.value;
        }
    });
    return JSON.stringify(data);
}

// 2. Automatikus esemény csatoló (Event Binder)
function bindAlakitElement(el) {
    if (el.__alakit_bound) return; // Már csatolva van

    let evAttr = el.getAttribute('alakit-event') || 'click';
    let cmd = el.getAttribute('alakit-cmd');
    let formAttr = el.getAttribute('alakit-form');

    // Bontunk (pl. 'keyup.enter' -> 'keyup' és 'enter')
    let [evType, evModifier] = evAttr.split('.');

    if (cmd) {
        el.addEventListener(evType, (e) => {
            // Automatikus prevent default <a> tageknél vagy submitnál
            if (el.tagName === 'A' || el.type === 'submit' || evType === 'submit') {
                e.preventDefault();
            }

            // Ha volt modifikátor (pl. enter), csak az arra billentyűre fusson
            if (evModifier && e.key && e.key.toLowerCase() !== evModifier.toLowerCase()) {
                return; // Kilépünk ha nem egyezik a gomb
            }

            let args = el.getAttribute('alakit-args') || '';

            // Ha form beküldésről van szó JSON adat kinyerése
            if (formAttr) {
                args = gatherFormData(formAttr);
            }
            // Ha sima checkbox
            else if (el.type === 'checkbox' && args === '') {
                args = el.checked.toString();
            }
            // Ha sima input/textarea és nincs args
            else if ((el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') && args === '') {
                args = el.value;
            }

            let msg = args ? `${cmd}|${args}` : `${cmd}|`;
            window.alakit(msg);
        });
        el.__alakit_bound = true;
    }
}

// 3. Kezdeti DOM és MutationObserver
document.addEventListener('DOMContentLoaded', () => {
    document.querySelectorAll('[alakit-cmd]').forEach(bindAlakitElement);

    const observer = new MutationObserver((mutations) => {
        mutations.forEach((mutation) => {
            if (mutation.type === 'childList') {
                mutation.addedNodes.forEach((node) => {
                    if (node.nodeType === 1) { // ELEMENT_NODE
                        if (node.hasAttribute('alakit-cmd')) {
                            bindAlakitElement(node);
                        }
                        node.querySelectorAll('[alakit-cmd]').forEach(bindAlakitElement);
                    }
                });
            }
        });
    });

    observer.observe(document.body, { childList: true, subtree: true });

    // --- ÚJ: Auto-seeding (Kezdeti szinkronizálás a DOM-ból) ---
    // Megkeressük az összes kezdeti értéket, amit a fejlesztő a HTML-be írt
    setTimeout(() => {
        document.querySelectorAll('[alakit-bind]').forEach(el => {
            const key = el.getAttribute('alakit-bind');
            const val = (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') ? 
                (el.type === 'checkbox' ? el.checked.toString() : el.value) : 
                el.innerText;
            
            if (val !== undefined && val !== null && val.toString().trim() !== "") {
                window.alakit(`alakit:init|{"key":"${key}","val":"${val.toString().trim()}"}`);
            }
        });
    }, 50); 
});

// 4. Reaktív Store Kezelő (State Management)
window.__alakit_store = {};

window.alakit_update_store = function (key, value) {
    // 1. Állapot mentése a memória JS reprezentációjába
    window.__alakit_store[key] = value;

    // Segédfüggvény: egy érték "igaz"-ságának (truthiness) eldöntése Rust-szemszögből
    const isTruthy = (val) => {
        if (typeof val === 'string') {
            const v = val.toLowerCase();
            return v === 'true' || v === '1' || v === 'yes' || v === 'on';
        }
        return !!val;
    };
    const truthyVal = isTruthy(value);

    // 2. Kötések (Bindings) feldolgozása: [alakit-bind="key"]
    document.querySelectorAll(`[alakit-bind='${key}']`).forEach(el => {
        if (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') {
            if (el.type === 'checkbox') {
                el.checked = truthyVal;
            } else {
                el.value = value;
            }
        } else {
            // Sima szöveges elemek (div, span, button)
            // Ha van 'alakit-html' attribútum, akkor HTML-ként renderelünk, különben biztonságos szövegként
            if (el.hasAttribute('alakit-html')) {
                el.innerHTML = value;
            } else {
                el.innerText = value;
            }
        }
    });

    // 3. Feltételes megjelenítés: [alakit-show="key"]
    document.querySelectorAll(`[alakit-show='${key}']`).forEach(el => {
        if (truthyVal) {
            el.style.display = ''; // Visszaáll az eredeti/alapértelmezett állapotra
        } else {
            el.style.display = 'none'; // Elrejtjük
        }
    });

    // 4. Feltételes elrejtés: [alakit-hide="key"]
    document.querySelectorAll(`[alakit-hide='${key}']`).forEach(el => {
        if (truthyVal) {
            el.style.display = 'none'; // Elrejtjük, ha IGAZ
        } else {
            el.style.display = ''; // Megjelenítjük
        }
    });
};

// 5. Toast Értesítési Rendszer (Notification API)
window.alakit_toast = function (type, message) {
    // Konténer lekérése vagy létrehozása
    let container = document.getElementById('alakit-toast-container');
    if (!container) {
        container = document.createElement('div');
        container.id = 'alakit-toast-container';
        document.body.appendChild(container);
    }

    // Új toast elem létrehozása
    const toast = document.createElement('div');
    toast.className = `alakit-toast ${type}`;

    // Ikon kiválasztása típus alapján
    let icon = 'ℹ️';
    if (type === 'success') icon = '✅';
    if (type === 'error') icon = '❌';
    if (type === 'warning') icon = '⚠️';

    toast.innerHTML = `<span style="font-size: 1.2rem;">${icon}</span> <span>${message}</span>`;

    // Hozzáadás a DOM-hoz
    container.appendChild(toast);

    // Automatikus eltűnés (autoclose) 3 másodperc múlva
    setTimeout(() => {
        toast.classList.add('fade-out');
        // Várjuk meg, amíg lefut a fade-out animáció (300ms)
        setTimeout(() => toast.remove(), 300);
    }, 3000);
};
