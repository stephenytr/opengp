use bcrypt::{hash, verify, DEFAULT_COST};
use opengp_domain::user::{PasswordError, PasswordHasher};

#[derive(Debug, Clone)]
pub struct BcryptPasswordHasher;

impl Default for BcryptPasswordHasher {
    fn default() -> Self {
        Self
    }
}

impl BcryptPasswordHasher {
    pub fn new() -> Self {
        Self
    }
}

impl PasswordHasher for BcryptPasswordHasher {
    fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        if password.is_empty() {
            return Err(PasswordError::EmptyPassword);
        }

        hash(password, DEFAULT_COST).map_err(|_| PasswordError::HashingFailed)
    }

    fn verify_password(&self, password_hash: &str, password: &str) -> Result<(), PasswordError> {
        let result =
            verify(password, password_hash).map_err(|_| PasswordError::VerificationFailed)?;
        if result {
            Ok(())
        } else {
            Err(PasswordError::VerificationFailed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_password_uses_unique_salt_for_same_input() {
        let hasher = BcryptPasswordHasher::new();

        let hash1 = hasher.hash_password("test123").unwrap();
        let hash2 = hasher.hash_password("test123").unwrap();

        // Same password with different salts produces different hashes
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn verify_password_succeeds_for_correct_password() {
        let hasher = BcryptPasswordHasher::new();
        let hash = hasher.hash_password("test123").unwrap();

        assert!(hasher.verify_password(&hash, "test123").is_ok());
    }

    #[test]
    fn verify_password_fails_for_wrong_password() {
        let hasher = BcryptPasswordHasher::new();
        let hash = hasher.hash_password("test123").unwrap();

        assert!(matches!(
            hasher.verify_password(&hash, "wrong"),
            Err(PasswordError::VerificationFailed)
        ));
    }
}

#[test]
fn print_hash() {
    let hasher = BcryptPasswordHasher::new();
    let hash = hasher.hash_password("test123").unwrap();
    println!("TEST123_HASH: {}", hash);
    let hash2 = hasher.hash_password("password").unwrap();
    println!("PASSWORD_HASH: {}", hash2);
}

#[test]
fn print_bannana_hash() {
    let hasher = BcryptPasswordHasher::new();
    let hash = hasher.hash_password("bannana").unwrap();
    println!("BANNANA_HASH: {}", hash);
}
