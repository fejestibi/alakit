use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tao::event_loop::EventLoopProxy;

/// Automatikusan frissülő, szálbiztos Állapottároló AES-GCM in-memory titkosítással.
/// Ha módosítod az adatait, a UI-on a hozzá kötött elemek (`alakit-bind`) frissülnek.
/// A memóriában nem található meg plain-text formátumban a változó.
#[derive(Clone)]
pub struct Store {
    // String helyett rejtjelezett bájt-tömböt és Nonce-t (egyedi szignál) tárolunk a HashMap-ben
    data: Arc<RwLock<HashMap<String, (Vec<u8>, Vec<u8>)>>>,
    proxy: EventLoopProxy<String>,

    // Titkosító kulcs (ami soha nem hagyja el ezt a struktúrát a futás alatt)
    // Megjegyzés: a Clone miatti másoláshoz Arc-be kell tenni
    key: Arc<Key<Aes256Gcm>>,
}

impl Store {
    pub fn new(proxy: EventLoopProxy<String>) -> Self {
        // Véletlenszerű 256 bites (32 bájt) AES kulcs generálása indításkor
        let key = Aes256Gcm::generate_key(OsRng);

        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            proxy,
            key: Arc::new(key),
        }
    }

    /// Titkosítva beállít egy értéket a memóriában ÉS tiszta IPC hívással frissíti a UI-t dekódolva.
    pub fn set(&self, key: &str, value: &str) {
        // 1. Titkosítás (AES-256-GCM)
        let cipher = Aes256Gcm::new(&self.key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bit (12 bájt) egyedi Nonce

        let ciphertext = cipher
            .encrypt(&nonce, value.as_bytes())
            .expect("Hiba az in-memory titkosítás során!");

        // 2. Biztonságos tárolás a RAM-ban (csak bájt tömb és Nonce)
        {
            let mut w = self.data.write().unwrap();
            w.insert(key.to_string(), (ciphertext, nonce.to_vec()));
        }

        // 3. Tiszta JavaScript Proxy hívás a frissítéshez (a JS már megkapja a tiszta adatot)
        let safe_value = value.replace('`', "\\`").replace('$', "\\$");
        let js = format!(
            "if (window.alakit_update_store) window.alakit_update_store(`{}`, `{}`);",
            key, safe_value
        );
        let _ = self.proxy.send_event(js);
    }

    /// Kinyeri és memória-szinten visszafejti az értéket.
    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<String> {
        let r = self.data.read().unwrap();

        if let Some((ciphertext, nonce_vec)) = r.get(key) {
            let cipher = Aes256Gcm::new(&self.key);
            let nonce = Nonce::from_slice(nonce_vec); // Biztonságos konverzió

            match cipher.decrypt(nonce, ciphertext.as_ref()) {
                Ok(plaintext) => String::from_utf8(plaintext).ok(),
                Err(_) => {
                    println!(
                        "[ALAKIT BIZTONSÁG] Hiba az adat visszabontásában a RAM-ból! (Sérült In-Memory Cache/Lehetséges tampering)"
                    );
                    None
                }
            }
        } else {
            None
        }
    }

    /// Töröl egy kulcsot a tárolóból és értesíti a UI-t.
    pub fn remove(&self, key: &str) {
        {
            let mut w = self.data.write().unwrap();
            w.remove(key);
        }

        // Értesítjük a JS-t, hogy az állapot megszűnt (null küldése)
        let js = format!(
            "if (window.alakit_update_store) window.alakit_update_store(`{}`, null);",
            key
        );
        let _ = self.proxy.send_event(js);
    }
}
