use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, SaltString},
    Argon2, PasswordHasher as ArgonPasswordHasher, PasswordVerifier,
};
use opengp_domain::user::{PasswordError, PasswordHasher};

#[derive(Debug, Clone)]
pub struct Argon2PasswordHasher {
    argon2: Argon2<'static>,
}

impl Default for Argon2PasswordHasher {
    fn default() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }
}

impl Argon2PasswordHasher {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PasswordHasher for Argon2PasswordHasher {
    fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        if password.is_empty() {
            return Err(PasswordError::EmptyPassword);
        }

        let salt = SaltString::generate(&mut OsRng);
        self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| PasswordError::HashingFailed)
    }

    fn verify_password(&self, password_hash: &str, password: &str) -> Result<(), PasswordError> {
        let parsed_hash =
            PasswordHash::new(password_hash).map_err(|_| PasswordError::InvalidHash)?;

        self.argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| PasswordError::VerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_password_uses_unique_salt_for_same_input() {
        let hasher = Argon2PasswordHasher::new();

        let hash1 = hasher.hash_password("test123").unwrap();
        let hash2 = hasher.hash_password("test123").unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn verify_password_succeeds_for_correct_password() {
        let hasher = Argon2PasswordHasher::new();
        let hash = hasher.hash_password("test123").unwrap();

        assert!(hasher.verify_password(&hash, "test123").is_ok());
    }

    #[test]
    fn verify_password_fails_for_wrong_password() {
        let hasher = Argon2PasswordHasher::new();
        let hash = hasher.hash_password("test123").unwrap();

        assert!(matches!(
            hasher.verify_password(&hash, "wrong"),
            Err(PasswordError::VerificationFailed)
        ));
    }
}
