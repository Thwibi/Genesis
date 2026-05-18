use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::digest::{digest, SHA256};
use ring::rand::{SecureRandom, SystemRandom};
use std::path::Path;
use tracing::info;

pub struct Vault;

impl Vault {
    pub fn initialize(key_path: &Path) -> Result<Self, VaultError> {
        if let Some(parent) = key_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| VaultError::IoError(e.to_string()))?;
        }
        if !key_path.exists() {
            info!(target: "vault", path = %key_path.display(), "Initializing vault with new master key");
            let key = generate_master_key()?;
            std::fs::write(key_path, &key)
                .map_err(|e| VaultError::IoError(e.to_string()))?;
            write_integrity_hash(key_path, &key)?;
        } else {
            info!(target: "vault", path = %key_path.display(), "Vault master key exists");
        }
        Ok(Self)
    }

    pub fn verify_integrity_at(key_path: &Path) -> Result<(), VaultError> {
        let key = std::fs::read(key_path)
            .map_err(|e| VaultError::IoError(e.to_string()))?;
        if key.len() != 32 {
            return Err(VaultError::InvalidKeyLength(key.len()));
        }
        let computed = hex::encode(digest(&SHA256, &key).as_ref());
        let hash_path = key_path.with_extension("bin.sha256");
        if hash_path.exists() {
            let stored = std::fs::read_to_string(&hash_path)
                .map_err(|e| VaultError::IoError(e.to_string()))?;
            let stored = stored.trim();
            if stored != computed {
                return Err(VaultError::IntegrityCheckFailed(
                    format!("Master key checksum mismatch: expected {}, got {}", stored, computed)
                ));
            }
        } else {
            write_integrity_hash(key_path, &key)?;
        }
        info!(target: "vault", "Vault integrity check passed");
        Ok(())
    }

    pub fn verify_integrity(&self) -> Result<(), VaultError> {
        Err(VaultError::IntegrityCheckFailed(
            "use verify_integrity_at(path) with explicit key path".to_string()
        ))
    }

    pub fn load_master_key(key_path: &Path) -> Result<Vec<u8>, VaultError> {
        let key = std::fs::read(key_path)
            .map_err(|e| VaultError::IoError(e.to_string()))?;

        if key.len() != 32 {
            return Err(VaultError::InvalidKeyLength(key.len()));
        }

        Ok(key)
    }

    /// Encrypt plaintext using AES-256-GCM.
    /// Output format: [nonce (12 bytes)][ciphertext][tag (16 bytes)]
    pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, VaultError> {
        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;
        let key = LessSafeKey::new(unbound_key);

        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = plaintext.to_vec();
        let tag_len = AES_256_GCM.tag_len();
        in_out.reserve(tag_len);

        key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;

        let mut result = Vec::with_capacity(12 + in_out.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&in_out);

        Ok(result)
    }

    /// Decrypt ciphertext using AES-256-GCM.
    /// Input format: [nonce (12 bytes)][ciphertext][tag (16 bytes)]
    pub fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, VaultError> {
        if ciphertext.len() < 12 + 16 {
            return Err(VaultError::InvalidCiphertext("ciphertext too short".to_string()));
        }

        let (nonce_bytes, encrypted) = ciphertext.split_at(12);
        let mut nonce_arr = [0u8; 12];
        nonce_arr.copy_from_slice(nonce_bytes);

        let unbound_key = UnboundKey::new(&AES_256_GCM, key)
            .map_err(|e| VaultError::CryptoError(e.to_string()))?;
        let key = LessSafeKey::new(unbound_key);

        let nonce = Nonce::assume_unique_for_key(nonce_arr);
        let mut in_out = encrypted.to_vec();

        let plaintext = key.open_in_place(nonce, Aad::empty(), &mut in_out)
            .map_err(|_| VaultError::DecryptionFailed)?;

        Ok(plaintext.to_vec())
    }
}

fn generate_master_key() -> Result<Vec<u8>, VaultError> {
    let rng = SystemRandom::new();
    let mut key = vec![0u8; 32];
    rng.fill(&mut key)
        .map_err(|e| VaultError::CryptoError(e.to_string()))?;
    Ok(key)
}

fn write_integrity_hash(key_path: &Path, key: &[u8]) -> Result<(), VaultError> {
    let hash = hex::encode(digest(&SHA256, key).as_ref());
    let hash_path = key_path.with_extension("bin.sha256");
    std::fs::write(&hash_path, hash)
        .map_err(|e| VaultError::IoError(e.to_string()))
}

#[derive(Debug)]
pub enum VaultError {
    CryptoError(String),
    IoError(String),
    InvalidKeyLength(usize),
    InvalidCiphertext(String),
    DecryptionFailed,
    IntegrityCheckFailed(String),
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            VaultError::IoError(msg) => write!(f, "IO error: {}", msg),
            VaultError::InvalidKeyLength(len) => write!(f, "Invalid key length: {}", len),
            VaultError::InvalidCiphertext(msg) => write!(f, "Invalid ciphertext: {}", msg),
            VaultError::DecryptionFailed => write!(f, "Decryption failed"),
            VaultError::IntegrityCheckFailed(msg) => write!(f, "Integrity check failed: {}", msg),
        }
    }
}

impl std::error::Error for VaultError {}
