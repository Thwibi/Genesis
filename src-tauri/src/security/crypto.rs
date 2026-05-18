use ring::digest::{digest, SHA256};
use ring::hkdf::{Salt, HKDF_SHA256};
use ring::rand::{SecureRandom, SystemRandom};

/// Derive a purpose-specific key from a master key using HKDF-SHA256.
pub fn derive_key(master: &[u8], salt: &[u8], info: &[u8], out_len: usize) -> Result<Vec<u8>, CryptoError> {
    let salt = Salt::new(HKDF_SHA256, salt);
    let prk = salt.extract(master);

    let mut okm = vec![0u8; out_len];
    prk.expand(&[info], HKDF_SHA256)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?
        .fill(&mut okm)
        .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;

    Ok(okm)
}

/// Generate cryptographically secure random bytes.
pub fn random_bytes(len: usize) -> Result<Vec<u8>, CryptoError> {
    let rng = SystemRandom::new();
    let mut bytes = vec![0u8; len];
    rng.fill(&mut bytes)
        .map_err(|e| CryptoError::RngError(e.to_string()))?;
    Ok(bytes)
}

/// Compute SHA-256 hash.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let result = digest(&SHA256, data);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(result.as_ref());
    hash
}

#[derive(Debug)]
pub enum CryptoError {
    KeyDerivationFailed(String),
    RngError(String),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::KeyDerivationFailed(msg) => write!(f, "Key derivation failed: {}", msg),
            CryptoError::RngError(msg) => write!(f, "RNG error: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}
