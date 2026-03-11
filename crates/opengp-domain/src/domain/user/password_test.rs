use uuid::Uuid;

use super::{PasswordError, PasswordHasher};

struct TestPasswordHasher;

impl PasswordHasher for TestPasswordHasher {
    fn hash_password(&self, password: &str) -> Result<String, PasswordError> {
        if password.is_empty() {
            return Err(PasswordError::EmptyPassword);
        }

        let salt = Uuid::new_v4();
        Ok(format!("{salt}${password}"))
    }

    fn verify_password(&self, password_hash: &str, password: &str) -> Result<(), PasswordError> {
        let Some((_, stored_password)) = password_hash.split_once('$') else {
            return Err(PasswordError::InvalidHash);
        };

        if stored_password == password {
            Ok(())
        } else {
            Err(PasswordError::VerificationFailed)
        }
    }
}

#[test]
fn password_hash_uses_unique_salts_for_same_password() {
    let hasher = TestPasswordHasher;

    let hash1 = hasher
        .hash_password("test123")
        .expect("first hash should succeed");
    let hash2 = hasher
        .hash_password("test123")
        .expect("second hash should succeed");

    assert_ne!(hash1, hash2, "hashes should differ due to random salt");
}

#[test]
fn password_verify_accepts_correct_password() {
    let hasher = TestPasswordHasher;
    let hash = hasher
        .hash_password("test123")
        .expect("hash should succeed");

    assert!(hasher.verify_password(&hash, "test123").is_ok());
}

#[test]
fn password_verify_rejects_incorrect_password() {
    let hasher = TestPasswordHasher;
    let hash = hasher
        .hash_password("test123")
        .expect("hash should succeed");

    assert!(matches!(
        hasher.verify_password(&hash, "wrong"),
        Err(PasswordError::VerificationFailed)
    ));
}
