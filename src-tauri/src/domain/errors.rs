use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("Session not found or invalid: {0}")]
    SessionNotFound(String),

    #[error("Session expired")]
    SessionExpired,

    #[error("Account locked")]
    AccountLocked,

    #[error("Account disabled")]
    AccountDisabled,

    #[error("Platform not found: {0}")]
    PlatformNotFound(String),

    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    #[error("Shift not found: {0}")]
    ShiftNotFound(String),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Audit write failed: {0}")]
    AuditWriteFailed(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Concurrent access detected")]
    ConcurrentAccessDetected,

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<DomainError> for crate::commands::CommandError {
    fn from(err: DomainError) -> Self {
        let code = match &err {
            DomainError::AuthenticationFailed(_) => "AUTH_FAILED",
            DomainError::AuthorizationFailed(_) => "AUTHORIZATION_FAILED",
            DomainError::SessionNotFound(_) => "SESSION_NOT_FOUND",
            DomainError::SessionExpired => "SESSION_EXPIRED",
            DomainError::AccountLocked => "ACCOUNT_LOCKED",
            DomainError::AccountDisabled => "ACCOUNT_DISABLED",
            DomainError::PlatformNotFound(_) => "PLATFORM_NOT_FOUND",
            DomainError::SecretNotFound(_) => "SECRET_NOT_FOUND",
            DomainError::ShiftNotFound(_) => "SHIFT_NOT_FOUND",
            DomainError::UserNotFound(_) => "USER_NOT_FOUND",
            DomainError::AuditWriteFailed(_) => "AUDIT_WRITE_FAILED",
            DomainError::EncryptionError(_) => "ENCRYPTION_ERROR",
            DomainError::ValidationError(_) => "VALIDATION_ERROR",
            DomainError::ConcurrentAccessDetected => "CONCURRENT_ACCESS",
            DomainError::DatabaseError(_) => "DATABASE_ERROR",
            DomainError::InternalError(_) => "INTERNAL_ERROR",
        };
        crate::commands::CommandError {
            code: code.to_string(),
            message: err.to_string(),
        }
    }
}
