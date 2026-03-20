use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

/// Read the 32-byte encryption key from `LLM_ENCRYPTION_KEY` env var (hex-encoded).
/// Shared with llm-provider — one key for all service credentials.
fn get_encryption_key() -> Result<[u8; 32]> {
    let hex_key = std::env::var("LLM_ENCRYPTION_KEY").map_err(|_| {
        anyhow!("LLM_ENCRYPTION_KEY environment variable not set. Generate one with: openssl rand -hex 32")
    })?;

    let bytes = hex::decode(&hex_key).map_err(|_| {
        anyhow!("LLM_ENCRYPTION_KEY must be a 64-character hex string (32 bytes)")
    })?;

    if bytes.len() != 32 {
        return Err(anyhow!(
            "LLM_ENCRYPTION_KEY must be exactly 32 bytes (64 hex chars), got {} bytes",
            bytes.len()
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&bytes);
    Ok(key)
}

pub fn encrypt_token(plaintext: &str) -> Result<String> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }

    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    #[allow(deprecated)]
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow!("Encryption failed: {}", e))?;

    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&combined))
}

pub fn decrypt_token(encrypted: &str) -> Result<String> {
    if encrypted.is_empty() {
        return Ok(String::new());
    }

    let key = get_encryption_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

    let combined = BASE64
        .decode(encrypted)
        .map_err(|e| anyhow!("Failed to decode base64: {}", e))?;

    if combined.len() < 12 {
        return Err(anyhow!("Encrypted data too short"));
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    #[allow(deprecated)]
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| anyhow!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| anyhow!("Decrypted data is not valid UTF-8: {}", e))
}

pub fn extract_token_prefix(token: &str) -> String {
    if token.is_empty() {
        "none".to_string()
    } else if token.len() <= 8 {
        format!("{}...", token)
    } else {
        format!("{}...", &token[..8])
    }
}

mod hex {
    pub fn decode(s: &str) -> Result<Vec<u8>, ()> {
        if s.len() % 2 != 0 {
            return Err(());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|_| ()))
            .collect()
    }
}
