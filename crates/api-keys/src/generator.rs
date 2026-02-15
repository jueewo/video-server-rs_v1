use rand::Rng;
use sha2::{Digest, Sha256};

/// Generate a secure random API key with the given prefix
///
/// Format: `{prefix}_{32_random_alphanumeric_chars}`
/// Example: `ak_live_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6`
pub fn generate_api_key(prefix: &str) -> String {
    const KEY_LENGTH: usize = 32;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";

    let mut rng = rand::thread_rng();
    let random_part: String = (0..KEY_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    format!("{}_{}", prefix, random_part)
}

/// Generate a production API key (ak_live_...)
pub fn generate_live_key() -> String {
    generate_api_key("ak_live")
}

/// Generate a test API key (ak_test_...)
pub fn generate_test_key() -> String {
    generate_api_key("ak_test")
}

/// Hash an API key using SHA-256
/// Returns hex-encoded hash string
pub fn hash_api_key(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Extract the display prefix from an API key
/// Returns first 12 characters for user-friendly display
///
/// Example: `ak_live_a1b2c3d4...` -> `ak_live_a1b2`
pub fn extract_prefix(key: &str) -> String {
    key.chars().take(12).collect()
}

/// Verify that a key matches its hash
pub fn verify_key(key: &str, hash: &str) -> bool {
    hash_api_key(key) == hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_live_key_format() {
        let key = generate_live_key();
        assert!(key.starts_with("ak_live_"));
        assert_eq!(key.len(), "ak_live_".len() + 32);
    }

    #[test]
    fn test_generate_test_key_format() {
        let key = generate_test_key();
        assert!(key.starts_with("ak_test_"));
        assert_eq!(key.len(), "ak_test_".len() + 32);
    }

    #[test]
    fn test_keys_are_unique() {
        let key1 = generate_live_key();
        let key2 = generate_live_key();
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_hash_is_deterministic() {
        let key = "ak_live_test123456789012345678901234";
        let hash1 = hash_api_key(key);
        let hash2 = hash_api_key(key);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_keys_different_hashes() {
        let key1 = "ak_live_test123456789012345678901234";
        let key2 = "ak_live_test123456789012345678901235";
        let hash1 = hash_api_key(key1);
        let hash2 = hash_api_key(key2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_extract_prefix() {
        let key = "ak_live_abc123def456ghi789jkl012mno345";
        let prefix = extract_prefix(key);
        assert_eq!(prefix, "ak_live_abc1");
    }

    #[test]
    fn test_verify_key() {
        let key = "ak_live_test123456789012345678901234";
        let hash = hash_api_key(key);
        assert!(verify_key(key, &hash));
        assert!(!verify_key("wrong_key", &hash));
    }
}
