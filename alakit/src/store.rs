use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tao::event_loop::EventLoopProxy;
use zeroize::{Zeroize, Zeroizing};

/// Biztonságos kulcs-tároló, amely zárolja a memóriát az mlock segítségével.
struct ProtectedKey {
    key: Key<Aes256Gcm>,
    _handle: Option<region::LockGuard>,
}

impl ProtectedKey {
    fn new(key: Key<Aes256Gcm>) -> Self {
        // Megpróbáljuk zárolni a memóriát, hogy ne kerüljön swap-ba (region 3.0)
        let handle = unsafe {
            region::lock(key.as_ptr(), key.len()).ok()
        };
        
        Self {
            key,
            _handle: handle,
        }
    }
}

// Biztosítjuk, hogy a kulcs törlődjön a memóriából, ha a struktúra felszabadul
impl Drop for ProtectedKey {
    fn drop(&mut self) {
        self.key.as_mut_slice().zeroize();
    }
}

/// Automatikusan frissülő, szálbiztos Állapottároló AES-GCM in-memory titkosítással.
/// Most már Zeroize és mlock támogatással a fokozott biztonság érdekében.
#[derive(Clone)]
pub struct Store {
    // A titkosított adatokat és nonce-okat is Zeroizing-olt pufferekben tároljuk
    data: Arc<RwLock<HashMap<String, (Zeroizing<Vec<u8>>, Zeroizing<Vec<u8>>)>>>,
    proxy: EventLoopProxy<String>,
    key: Arc<ProtectedKey>,
}

impl Store {
    pub fn new(proxy: EventLoopProxy<String>) -> Self {
        let key_raw = Aes256Gcm::generate_key(OsRng);
        
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            proxy,
            key: Arc::new(ProtectedKey::new(key_raw)),
        }
    }

    /// Titkosítva beállít egy értéket a memóriában ÉS tiszta IPC hívással frissíti a UI-t dekódolva.
    pub fn set(&self, key: &str, value: &str) {
        let cipher = Aes256Gcm::new(&self.key.key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // A plaintext értéket Zeroizing burkolóba tesszük, hogy használat után törlődjön
        let plaintext = Zeroizing::new(value.as_bytes().to_vec());
        
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .expect("Hiba az in-memory titkosítás során!");

        {
            let mut w = self.data.write().unwrap();
            w.insert(
                key.to_string(), 
                (Zeroizing::new(ciphertext), Zeroizing::new(nonce.to_vec()))
            );
        }

        let safe_value = value.replace('`', "\\`").replace('$', "\\$");
        let js = format!(
            "if (window.alakit_update_store) window.alakit_update_store(`{}`, `{}`);",
            key, safe_value
        );
        let _ = self.proxy.send_event(js);
    }

    /// Kinyeri és memória-szinten visszafejti az értéket.
    pub fn get(&self, key: &str) -> Option<String> {
        let r = self.data.read().unwrap();

        if let Some((ciphertext, nonce_vec)) = r.get(key) {
            let cipher = Aes256Gcm::new(&self.key.key);
            let nonce = Nonce::from_slice(nonce_vec.as_ref());

            match cipher.decrypt(nonce, ciphertext.as_ref()) {
                Ok(plaintext_bytes) => {
                    // A visszafejtett bájtokat is Zeroizing-ba tesszük menet közben
                    let plaintext_wrap = Zeroizing::new(plaintext_bytes);
                    String::from_utf8(plaintext_wrap.to_vec()).ok()
                }
                Err(_) => {
                    println!(
                        "[ALAKIT BIZTONSÁG] Hiba az adat visszabontásában a RAM-ból!"
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
            // A HashMap-ből való eltávolításkor a Zeroizing drop-ja automatikusan törli a memóriát
            w.remove(key);
        }

        let js = format!(
            "if (window.alakit_update_store) window.alakit_update_store(`{}`, null);",
            key
        );
        let _ = self.proxy.send_event(js);
    }
}
