use argon2::password_hash::{rand_core::OsRng, PasswordHash, SaltString};
use argon2::password_hash::PasswordVerifier;
use argon2::PasswordHasher;
use argon2::Argon2;

pub struct PasswordHasherService;

impl PasswordHasherService {
    /// Hash a password using Argon2id with OWASP-recommended minimum parameters.
    ///
    /// Current parameters:
    /// - Algorithm: Argon2id
    /// - Memory: 19 MiB (19456 KiB)
    /// - Iterations: 2
    /// - Parallelism: 1
    pub fn hash(password: &str) -> Result<String, PasswordError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| PasswordError::HashingFailed(e.to_string()))?;

        Ok(hash.to_string())
    }

    /// Verify a password against an Argon2id hash string.
    pub fn verify(password: &str, hash: &str) -> Result<bool, PasswordError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| PasswordError::VerificationFailed(e.to_string()))?;

        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}

#[derive(Debug)]
pub enum PasswordError {
    HashingFailed(String),
    VerificationFailed(String),
}

impl std::fmt::Display for PasswordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordError::HashingFailed(msg) => write!(f, "Password hashing failed: {}", msg),
            PasswordError::VerificationFailed(msg) => write!(f, "Password verification failed: {}", msg),
        }
    }
}

impl std::error::Error for PasswordError {}
