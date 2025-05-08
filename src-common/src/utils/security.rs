use base64::{engine::general_purpose::STANDARD, Engine};
use ring::aead::{Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, CHACHA20_POLY1305};
use ring::error::Unspecified;
use ring::rand::{SecureRandom, SystemRandom};
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::config::config_path;

const NONCE_LEN: usize = 12;
const SALT_LEN: usize = 16;
const KEY_LEN: usize = 32;
const SALT_FILE: &str = "salt.bin";

/// Derive a key using PBKDF2
fn derive_key() -> Result<[u8; KEY_LEN], String> {
    let salt_path = config_path(SALT_FILE);
    let mut salt = [0u8; SALT_LEN];
    
    // Create or load salt
    if !salt_path.exists() {
        // Generate new salt
        let rng = SystemRandom::new();
        rng.fill(&mut salt)
            .map_err(|_| "Failed to generate random salt".to_string())?;
            
        // Save salt
        std::fs::write(&salt_path, &salt)
            .map_err(|e| format!("Failed to save salt: {}", e))?;
    } else {
        // Load existing salt
        let salt_data = std::fs::read(&salt_path)
            .map_err(|e| format!("Failed to read salt: {}", e))?;
            
        if salt_data.len() != SALT_LEN {
            return Err("Invalid salt length".to_string());
        }
        
        salt.copy_from_slice(&salt_data);
    }
    
    // Use application-specific "password" for encryption
    // This is not intended for high security, just basic obfuscation of API keys
    let app_key = format!("MCP_CLIENT_{}", std::env::consts::OS);
    
    // Derive key using PBKDF2
    let mut key = [0u8; KEY_LEN];
    ring::pbkdf2::derive(
        ring::pbkdf2::PBKDF2_HMAC_SHA256,
        NonZeroU32::new(100_000).unwrap(),
        &salt,
        app_key.as_bytes(),
        &mut key,
    );
    
    Ok(key)
}

/// Simple nonce sequence for encryption
struct CounterNonceSequence {
    counter: AtomicU32,
}

impl CounterNonceSequence {
    fn new(initial: u32) -> Self {
        Self {
            counter: AtomicU32::new(initial),
        }
    }
}

impl NonceSequence for CounterNonceSequence {
    fn advance(&self) -> Result<Nonce, Unspecified> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        let counter = self.counter.fetch_add(1, Ordering::Relaxed);
        
        for (i, byte) in counter.to_be_bytes().iter().take(4).enumerate() {
            nonce_bytes[i] = *byte;
        }
        
        Nonce::try_assume_unique_for_key(&nonce_bytes)
    }
}

/// Encrypt data
pub fn encrypt(plaintext: &str) -> Result<Vec<u8>, String> {
    let key = derive_key()?;
    let rng = SystemRandom::new();
    
    // Generate random initial counter
    let mut counter_bytes = [0u8; 4];
    rng.fill(&mut counter_bytes)
        .map_err(|_| "Failed to generate random counter".to_string())?;
    let counter = u32::from_be_bytes(counter_bytes);
    
    // Create encryption key
    let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &key)
        .map_err(|_| "Failed to create encryption key".to_string())?;
    let nonce_sequence = CounterNonceSequence::new(counter);
    let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);
    
    // Encrypt the data
    let mut in_out = plaintext.as_bytes().to_vec();
    sealing_key
        .seal_in_place_append_tag(Aad::empty(), &mut in_out)
        .map_err(|_| "Encryption failed".to_string())?;
    
    // Prepend the counter bytes for decryption
    let mut result = counter_bytes.to_vec();
    result.extend_from_slice(&in_out);
    
    Ok(result)
}

/// Decrypt data
pub fn decrypt(ciphertext: &[u8]) -> Result<String, String> {
    if ciphertext.len() < 4 + 16 {
        // 4 bytes counter + at least 16 bytes ciphertext (tag)
        return Err("Invalid ciphertext length".to_string());
    }
    
    let key = derive_key()?;
    
    // Extract counter
    let mut counter_bytes = [0u8; 4];
    counter_bytes.copy_from_slice(&ciphertext[0..4]);
    let counter = u32::from_be_bytes(counter_bytes);
    
    // Create decryption key
    let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &key)
        .map_err(|_| "Failed to create decryption key".to_string())?;
    let nonce_sequence = CounterNonceSequence::new(counter);
    let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);
    
    // Decrypt the data
    let mut in_out = ciphertext[4..].to_vec();
    let plaintext = opening_key
        .open_in_place(Aad::empty(), &mut in_out)
        .map_err(|_| "Decryption failed".to_string())?;
    
    String::from_utf8(plaintext.to_vec())
        .map_err(|_| "Invalid UTF-8 in decrypted data".to_string())
}
