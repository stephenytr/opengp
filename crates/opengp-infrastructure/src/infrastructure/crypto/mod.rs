//! Encryption module
//!
//! Provides AES-256-GCM encryption for sensitive data fields.
//! Encryption is application-level and transparent to the database.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use opengp_infrastructure::infrastructure::crypto::EncryptionService;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let crypto = EncryptionService::new()?;
//!
//! // Encrypt sensitive data
//! let plaintext = "Patient clinical notes";
//! let ciphertext = crypto.encrypt(plaintext)?;
//!
//! // Decrypt when needed
//! let decrypted = crypto.decrypt(&ciphertext)?;
//! assert_eq!(plaintext, decrypted);
//! # Ok(())
//! # }
//! ```
//!
//! ## Security Notes
//!
//! - Uses AES-256-GCM (authenticated encryption)
//! - Random nonce generated for each encryption
//! - Nonce prepended to ciphertext for storage
//! - Key loaded from ENCRYPTION_KEY environment variable
//! - Key must be 32 bytes (64 hex characters)

pub mod password;
pub use password::BcryptPasswordHasher;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;

/// Encryption service for sensitive data
///
/// Provides AES-256-GCM encryption with random nonces.
/// Key is loaded from environment variable on initialization.
pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl std::fmt::Debug for EncryptionService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncryptionService")
            .field("cipher", &"<redacted>")
            .finish()
    }
}

impl EncryptionService {
    /// Initialize encryption service with key from environment
    ///
    /// Loads encryption key from `ENCRYPTION_KEY` environment variable.
    /// Key must be hex-encoded and 32 bytes (64 hex characters).
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - `ENCRYPTION_KEY` environment variable is not set
    /// - Key is not valid hex
    /// - Key is not 32 bytes
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use opengp_infrastructure::infrastructure::crypto::EncryptionService;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// std::env::set_var("ENCRYPTION_KEY", "0".repeat(64));
    /// let service = EncryptionService::new()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self, CryptoError> {
        let key_bytes = Self::load_key_from_env()?;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| CryptoError::InvalidKey)?;

        Ok(Self { cipher })
    }

    /// Initialize encryption service with provided key
    ///
    /// Creates an encryption service with an explicitly provided key.
    /// Key must be hex-encoded and 32 bytes (64 hex characters).
    ///
    /// # Arguments
    ///
    /// * `key_hex` - Hex-encoded encryption key (must be 64 characters for 32 bytes)
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Key is not valid hex
    /// - Key is not 32 bytes
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use opengp_infrastructure::infrastructure::crypto::EncryptionService;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let key_hex = "0".repeat(64);
    /// let service = EncryptionService::new_with_key(&key_hex)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_with_key(key_hex: &str) -> Result<Self, CryptoError> {
        let key_bytes = Self::decode_key(key_hex)?;
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| CryptoError::InvalidKey)?;

        Ok(Self { cipher })
    }

    /// Encrypt sensitive data
    ///
    /// Encrypts plaintext using AES-256-GCM with a random nonce.
    /// The nonce is prepended to the ciphertext for storage.
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt
    ///
    /// # Returns
    ///
    /// Encrypted data as bytes with format: `[nonce (12 bytes)][ciphertext][tag (16 bytes)]`
    ///
    /// # Errors
    ///
    /// Returns error if encryption fails (should not occur in normal operation).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use opengp_infrastructure::infrastructure::crypto::EncryptionService;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # std::env::set_var("ENCRYPTION_KEY", "0".repeat(64));
    /// let crypto = EncryptionService::new()?;
    /// let encrypted = crypto.encrypt("sensitive data")?;
    /// assert!(encrypted.len() > 12 + 16);  // nonce + tag + data
    /// # Ok(())
    /// # }
    /// ```
    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, CryptoError> {
        // Generate random 12-byte nonce
        let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt plaintext
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Prepend nonce to ciphertext for storage
        // Format: [12 bytes nonce][ciphertext + 16 bytes tag]
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);

        Ok(result)
    }

    /// Decrypt sensitive data
    ///
    /// Decrypts data that was encrypted with `encrypt()`.
    /// Expects format: `[nonce (12 bytes)][ciphertext][tag (16 bytes)]`
    ///
    /// # Arguments
    ///
    /// * `encrypted` - Encrypted data (must include prepended nonce)
    ///
    /// # Returns
    ///
    /// Decrypted plaintext as UTF-8 string
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Data is too short (less than 12 bytes)
    /// - Decryption fails (authentication tag invalid, wrong key, corrupted data)
    /// - Result is not valid UTF-8
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use opengp_infrastructure::infrastructure::crypto::EncryptionService;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # std::env::set_var("ENCRYPTION_KEY", "0".repeat(64));
    /// let crypto = EncryptionService::new()?;
    /// let plaintext = "sensitive data";
    /// let encrypted = crypto.encrypt(plaintext)?;
    /// let decrypted = crypto.decrypt(&encrypted)?;
    /// assert_eq!(plaintext, decrypted);
    /// # Ok(())
    /// # }
    /// ```
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<String, CryptoError> {
        // Validate minimum length (12 bytes nonce + 16 bytes tag)
        if encrypted.len() < 12 {
            return Err(CryptoError::InvalidCiphertext);
        }

        // Extract nonce (first 12 bytes)
        let nonce = Nonce::from_slice(&encrypted[..12]);

        // Decrypt remaining bytes (ciphertext + tag)
        let plaintext = self
            .cipher
            .decrypt(nonce, &encrypted[12..])
            .map_err(|_| CryptoError::DecryptionFailed)?;

        // Convert to UTF-8 string
        String::from_utf8(plaintext).map_err(|_| CryptoError::InvalidUtf8)
    }

    /// Load encryption key from environment variable
    ///
    /// Reads `ENCRYPTION_KEY` environment variable and decodes from hex.
    fn load_key_from_env() -> Result<Vec<u8>, CryptoError> {
        let key_hex = std::env::var("ENCRYPTION_KEY").map_err(|_| CryptoError::MissingKey)?;
        Self::decode_key(&key_hex)
    }

    /// Decode and validate hex-encoded encryption key
    ///
    /// Decodes a hex string to bytes and validates it's 32 bytes for AES-256.
    fn decode_key(key_hex: &str) -> Result<Vec<u8>, CryptoError> {
        // Decode hex string to bytes
        let key_bytes = hex::decode(key_hex).map_err(|_| CryptoError::InvalidKeyFormat)?;

        // Validate key length (must be 32 bytes for AES-256)
        if key_bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength(key_bytes.len()));
        }

        Ok(key_bytes)
    }
}

/// Encryption errors
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Missing encryption key - set ENCRYPTION_KEY environment variable")]
    MissingKey,

    #[error("Invalid key format - key must be hex-encoded")]
    InvalidKeyFormat,

    #[error("Invalid key length: expected 32 bytes, got {0} bytes")]
    InvalidKeyLength(usize),

    #[error("Invalid key - failed to initialize cipher")]
    InvalidKey,

    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed - authentication tag invalid, wrong key, or corrupted data")]
    DecryptionFailed,

    #[error("Invalid ciphertext - data too short")]
    InvalidCiphertext,

    #[error("Decrypted data is not valid UTF-8")]
    InvalidUtf8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mutex to serialize tests that mutate ENCRYPTION_KEY environment variable.
    /// Required because std::env::set_var is not thread-safe across parallel tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Generate a random 32-byte key for testing
    fn generate_test_key() -> String {
        let key: [u8; 32] = rand::thread_rng().gen();
        hex::encode(key)
    }

    #[test]
    fn test_new_with_valid_key() {
        let _guard = ENV_LOCK.lock().unwrap();
        let key = generate_test_key();
        std::env::set_var("ENCRYPTION_KEY", &key);

        let result = EncryptionService::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_without_key() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("ENCRYPTION_KEY");

        let result = EncryptionService::new();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::MissingKey));
    }

    #[test]
    fn test_new_with_invalid_hex() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("ENCRYPTION_KEY", "not_hex_data");

        let result = EncryptionService::new();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::InvalidKeyFormat));
    }

    #[test]
    fn test_new_with_wrong_length_key() {
        let _guard = ENV_LOCK.lock().unwrap();
        // 16 bytes instead of 32
        let key = hex::encode([0u8; 16]);
        std::env::set_var("ENCRYPTION_KEY", &key);

        let result = EncryptionService::new();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CryptoError::InvalidKeyLength(16)
        ));
    }

    #[test]
    fn test_encrypt_decrypt() {
        let _guard = ENV_LOCK.lock().unwrap();
        let key = generate_test_key();
        std::env::set_var("ENCRYPTION_KEY", &key);

        let crypto = EncryptionService::new().unwrap();
        let plaintext = "sensitive patient data";

        // Encrypt
        let encrypted = crypto.encrypt(plaintext).unwrap();

        // Should be longer than plaintext (nonce + tag)
        assert!(encrypted.len() > plaintext.len());

        // Decrypt
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let _guard = ENV_LOCK.lock().unwrap();
        let key = generate_test_key();
        std::env::set_var("ENCRYPTION_KEY", &key);

        let crypto = EncryptionService::new().unwrap();
        let plaintext = "test data";

        // Encrypt same plaintext twice
        let encrypted1 = crypto.encrypt(plaintext).unwrap();
        let encrypted2 = crypto.encrypt(plaintext).unwrap();

        // Should produce different ciphertext due to random nonce
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt to same plaintext
        assert_eq!(crypto.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(crypto.decrypt(&encrypted2).unwrap(), plaintext);
    }

    #[test]
    fn test_decrypt_with_wrong_key() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Encrypt with one key
        let key1 = generate_test_key();
        std::env::set_var("ENCRYPTION_KEY", &key1);
        let crypto1 = EncryptionService::new().unwrap();
        let encrypted = crypto1.encrypt("secret").unwrap();

        // Try to decrypt with different key
        let key2 = generate_test_key();
        std::env::set_var("ENCRYPTION_KEY", &key2);
        let crypto2 = EncryptionService::new().unwrap();

        let result = crypto2.decrypt(&encrypted);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::DecryptionFailed));
    }

    #[test]
    fn test_decrypt_invalid_ciphertext() {
        let _guard = ENV_LOCK.lock().unwrap();
        let key = generate_test_key();
        std::env::set_var("ENCRYPTION_KEY", &key);
        let crypto = EncryptionService::new().unwrap();

        // Too short
        let result = crypto.decrypt(&[0u8; 10]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CryptoError::InvalidCiphertext
        ));

        // Corrupted data
        let mut encrypted = crypto.encrypt("test").unwrap();
        encrypted[15] ^= 0xFF; // Flip bits in ciphertext
        let result = crypto.decrypt(&encrypted);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::DecryptionFailed));
    }

    #[test]
    fn test_encrypt_empty_string() {
        let _guard = ENV_LOCK.lock().unwrap();
        let key = generate_test_key();
        std::env::set_var("ENCRYPTION_KEY", &key);
        let crypto = EncryptionService::new().unwrap();

        let encrypted = crypto.encrypt("").unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!("", decrypted);
    }

    #[test]
    fn test_encrypt_unicode() {
        let _guard = ENV_LOCK.lock().unwrap();
        let key = generate_test_key();
        std::env::set_var("ENCRYPTION_KEY", &key);
        let crypto = EncryptionService::new().unwrap();

        let plaintext = "Test with émojis 🔒 and ünïcödë";
        let encrypted = crypto.encrypt(plaintext).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_long_text() {
        let _guard = ENV_LOCK.lock().unwrap();
        let key = generate_test_key();
        std::env::set_var("ENCRYPTION_KEY", &key);
        let crypto = EncryptionService::new().unwrap();

        // Test with clinical notes length data
        let plaintext = "S: Patient presents with headache for 3 days. ".repeat(50);
        let encrypted = crypto.encrypt(&plaintext).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!(plaintext, decrypted);
    }
}
