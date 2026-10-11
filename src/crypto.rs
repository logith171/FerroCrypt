use aes_gcm::{
    // Add AeadCore to this list:
    aead::{Aead, AeadCore, KeyInit, OsRng}, 
    Aes256Gcm, Nonce,
};
use argon2::Argon2;

pub struct Crypto;

impl Crypto {
    pub fn derive_key(password: &str) -> [u8; 32] {
        let salt = b"ferrocrypt_secure_salt_2024"; 
        let mut key = [0u8; 32];
        let argon2 = Argon2::default();
        let _ = argon2.hash_password_into(password.as_bytes(), salt, &mut key);
        key
    }

    pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
        let cipher = Aes256Gcm::new(key.into());
        
        // This is the line that was failing:
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); 
        
        let mut ciphertext = cipher
            .encrypt(&nonce, data)
            .expect("Encryption failed!");

        ciphertext.extend_from_slice(&nonce);
        ciphertext
    }

    pub fn decrypt(encrypted_data: &[u8], key: &[u8; 32]) -> Option<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return None;
        }
        let (ciphertext, nonce_slice) = encrypted_data.split_at(encrypted_data.len() - 12);
        let nonce = Nonce::from_slice(nonce_slice);
        let cipher = Aes256Gcm::new(key.into());
        cipher.decrypt(nonce, ciphertext).ok()
    }
}