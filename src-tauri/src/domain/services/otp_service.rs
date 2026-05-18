use std::sync::Arc;
use totp_rs::{Algorithm, TOTP};
use tracing::info;
use uuid::Uuid;

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

pub struct OtpService {
    db_pool: Arc<DatabasePool>,
    audit_logger: Arc<AuditLogger>,
    vault_key: SecretBytes,
}

impl OtpService {
    pub fn new(db_pool: Arc<DatabasePool>, audit_logger: Arc<AuditLogger>, master_key: &[u8]) -> Self {
        let vault_key = derive_key(master_key, VAULT_KEY_SALT, VAULT_KEY_INFO, 32)
            .expect("Failed to derive vault encryption key");
        Self {
            db_pool,
            audit_logger,
            vault_key: SecretBytes::new(vault_key),
        }
    }

    pub async fn generate_otp(
        &self,
        session_id: Uuid,
        platform_id: Uuid,
    ) -> Result<String, DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Validate session
        let session = SessionRepo::find_by_id(&*conn, session_id)
            .map_err(|_| DomainError::SessionNotFound(session_id.to_string()))?;

        if session.revoked_at.is_some() {
            return Err(DomainError::SessionExpired);
        }

        // Retrieve encrypted secret
        let mfa_secret = SecretRepo::find_active_by_platform(&*conn, platform_id)
            .map_err(|_| DomainError::SecretNotFound(platform_id.to_string()))?;

        // Decrypt in memory using derived vault key
        let decrypted = Vault::decrypt(self.vault_key.as_ref(), &mfa_secret.encrypted_secret)
            .map_err(|e| DomainError::EncryptionError(e.to_string()))?;

        let secret_bytes = SecretBytes::new(decrypted);

        // Generate TOTP
        let totp = TOTP::new(
            Algorithm::SHA1,
            mfa_secret.digits as usize,
            1,
            mfa_secret.period,
            secret_bytes.as_ref().to_vec(),
            None,
            String::new(),
        )
        .map_err(|e| DomainError::InternalError(format!("TOTP generation failed: {}", e)))?;

        let otp = totp.generate_current()
            .map_err(|e| DomainError::InternalError(format!("TOTP generation failed: {}", e)))?;

        // SECURITY: SecretBytes zeroizes on drop here

        // Audit OTP generation
        let event = AuditEventBuilder::new(AuditEventType::OtpGenerated)
            .with_user_id(session.user_id)
            .with_session_id(session_id)
            .with_platform_id(platform_id)
            .with_workstation_id(&session.workstation_id)
            .with_metadata(serde_json::json!({
                "platform_id": platform_id.to_string(),
            }))
            .build();

        self.audit_logger.write(event).await
            .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

        info!(target: "otp", session_id = %session_id, platform_id = %platform_id, "OTP generated");

        Ok(otp)
    }

    pub async fn copy_otp_to_clipboard(
        &self,
        session_id: Uuid,
        platform_id: Uuid,
        otp: String,
    ) -> Result<(), DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let session = SessionRepo::find_by_id(&*conn, session_id)
            .map_err(|_| DomainError::SessionNotFound(session_id.to_string()))?;

        // SECURITY: Write audit event BEFORE clipboard operation
        let event = AuditEventBuilder::new(AuditEventType::OtpCopied)
            .with_user_id(session.user_id)
            .with_session_id(session_id)
            .with_platform_id(platform_id)
            .with_workstation_id(&session.workstation_id)
            .with_metadata(serde_json::json!({
                "clipboard_timeout_secs": 30,
                "otp_length": otp.len(),
            }))
            .build();

        self.audit_logger.write(event).await
            .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

        // Clipboard write handled by Tauri command layer with auto-clear
        info!(target: "otp", session_id = %session_id, platform_id = %platform_id, "OTP copied to clipboard");

        Ok(())
    }
}
