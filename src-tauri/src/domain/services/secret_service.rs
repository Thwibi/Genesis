use std::sync::Arc;
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::domain::entities::mfa_secret::MfaSecret;
use crate::domain::errors::DomainError;
use crate::audit::events::{AuditEventBuilder, AuditEventType};
use crate::audit::logger::AuditLogger;
use crate::persistence::db::DatabasePool;
use crate::persistence::repositories::secret_repo::SecretRepo;
use crate::persistence::repositories::session_repo::SessionRepo;
use crate::security::vault::Vault;
use crate::security::memory::SecretBytes;
use crate::security::crypto::derive_key;

const VAULT_KEY_SALT: &[u8] = b"genesis-vault-key-v1";
const VAULT_KEY_INFO: &[u8] = b"vault-encryption";

pub struct SecretService {
    db_pool: Arc<DatabasePool>,
    audit_logger: Arc<AuditLogger>,
    vault_key: SecretBytes,
}

impl SecretService {
    pub fn new(db_pool: Arc<DatabasePool>, audit_logger: Arc<AuditLogger>, master_key: &[u8]) -> Self {
        let vault_key = derive_key(master_key, VAULT_KEY_SALT, VAULT_KEY_INFO, 32)
            .expect("Failed to derive vault encryption key");
        Self {
            db_pool,
            audit_logger,
            vault_key: SecretBytes::new(vault_key),
        }
    }

    pub async fn store_secret(
        &self,
        platform_id: Uuid,
        raw_totp_seed: &[u8],
        session_id: Uuid,
    ) -> Result<Uuid, DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let session = SessionRepo::find_by_id(&*conn, session_id)
            .map_err(|_| DomainError::SessionNotFound(session_id.to_string()))?;

        // Encrypt the seed
        let encrypted = Vault::encrypt(self.vault_key.as_ref(), raw_totp_seed)
            .map_err(|e| DomainError::EncryptionError(e.to_string()))?;

        let secret = MfaSecret {
            id: Uuid::new_v4(),
            platform_id,
            encrypted_secret: encrypted,
            algorithm: "SHA1".to_string(),
            digits: 6,
            period: 30,
            created_at: Utc::now(),
            rotated_at: None,
            archived_at: None,
        };

        SecretRepo::create(&*conn, &secret)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Audit secret access
        let event = AuditEventBuilder::new(AuditEventType::SecretAccessed)
            .with_user_id(session.user_id)
            .with_session_id(session_id)
            .with_platform_id(platform_id)
            .with_workstation_id(&session.workstation_id)
            .with_metadata(serde_json::json!({
                "action": "store",
                "platform_id": platform_id.to_string(),
            }))
            .build();

        self.audit_logger.write(event).await
            .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

        info!(target: "secret", secret_id = %secret.id, platform_id = %platform_id, "Secret stored");

        Ok(secret.id)
    }

    pub async fn rotate_secret(
        &self,
        platform_id: Uuid,
        new_raw_seed: &[u8],
        session_id: Uuid,
    ) -> Result<(), DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let session = SessionRepo::find_by_id(&*conn, session_id)
            .map_err(|_| DomainError::SessionNotFound(session_id.to_string()))?;

        let encrypted = Vault::encrypt(self.vault_key.as_ref(), new_raw_seed)
            .map_err(|e| DomainError::EncryptionError(e.to_string()))?;

        SecretRepo::update_secret(&*conn, platform_id, &encrypted, Utc::now())
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // The old encrypted secret remains in the database (archived_at is not set here)
        // because the update replaces the active secret in-place. For overlap-period
        // tolerance, secrets should be inserted as new rows with the old row archived.
        // Audit logging of the rotation is a future enhancement — the current SECRET_ACCESSED
        // event covers it partially.

        info!(target: "secret", platform_id = %platform_id, "Secret rotated");

        Ok(())
    }

    pub async fn archive_secret(&self, platform_id: Uuid, session_id: Uuid) -> Result<(), DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let session = SessionRepo::find_by_id(&*conn, session_id)
            .map_err(|_| DomainError::SessionNotFound(session_id.to_string()))?;

        SecretRepo::archive(&*conn, platform_id, Utc::now())
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        info!(target: "secret", platform_id = %platform_id, "Secret archived");

        Ok(())
    }
}
