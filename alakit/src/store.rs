use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tao::event_loop::EventLoopProxy;
use zeroize::{Zeroize, Zeroizing};

/// Secure key store that locks memory using mlock.
struct ProtectedKey {
    key: Key<Aes256Gcm>,
    _handle: Option<region::LockGuard>,
}

impl ProtectedKey {
    fn new(key: Key<Aes256Gcm>) -> Self {
        // Lock memory region (mlock) against swapping (region 3.0)
        let handle = unsafe {
            region::lock(key.as_ptr(), key.len()).ok()
        };
        
        Self {
            key,
            _handle: handle,
        }
    }
}

// Automatic data clearing (zeroize) on memory deallocation
impl Drop for ProtectedKey {
    fn drop(&mut self) {
        self.key.as_mut_slice().zeroize();
    }
}

/// Thread-safe State Store with AES-GCM in-memory encryption,
/// Zeroize and mlock memory protection.
#[derive(Clone)]
pub struct Store {
    // Store cryptographic data in Zeroizing buffers
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

    /// Set value encrypted, then update UI state.
    pub fn set(&self, key: &str, value: &str) {
        let cipher = Aes256Gcm::new(&self.key.key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Temporary plaintext buffer with secure clearing
        let plaintext = Zeroizing::new(value.as_bytes().to_vec());
        
        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .expect("In-memory encryption failed!");

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

    /// Extracts and decrypts the value at memory level.
    pub fn get(&self, key: &str) -> Option<String> {
        let r = self.data.read().unwrap();

        if let Some((ciphertext, nonce_vec)) = r.get(key) {
            let cipher = Aes256Gcm::new(&self.key.key);
            let nonce = Nonce::from_slice(nonce_vec.as_ref());

            match cipher.decrypt(nonce, ciphertext.as_ref()) {
                Ok(plaintext_bytes) => {
                    // Protection of decrypted bytes
                    let plaintext_wrap = Zeroizing::new(plaintext_bytes);
                    String::from_utf8(plaintext_wrap.to_vec()).ok()
                }
                Err(_) => {
                    println!(
                        "[ALAKIT SECURITY] Error decrypting data from RAM!"
                    );
                    None
                }
            }
        } else {
            None
        }
    }

    /// Removes a key from the store and notifies the UI.
    pub fn remove(&self, key: &str) {
        {
            let mut w = self.data.write().unwrap();
            // Removal from the HashMap triggers the Zeroizing drop method
            w.remove(key);
        }

        let js = format!(
            "if (window.alakit_update_store) window.alakit_update_store(`{}`, null);",
            key
        );
        let _ = self.proxy.send_event(js);
    }
}
