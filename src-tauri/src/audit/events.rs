use chrono::Utc;
use uuid::Uuid;

use crate::domain::entities::audit_event::AuditEvent;

#[derive(Debug, Clone, PartialEq)]
pub enum AuditEventType {
    LoginSuccess,
    LoginFailure,
    OtpGenerated,
    OtpCopied,
    SecretAccessed,
    SessionRevoked,
    PermissionChanged,
    ShiftAssignmentChanged,
    SuspiciousActivityDetected,
    FailedAccessAttempt,
    ClipboardCleared,
    ClipboardTimeout,
    PlatformCreated,
    PlatformUpdated,
    PlatformDeleted,
}

impl AuditEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditEventType::LoginSuccess => "LOGIN_SUCCESS",
            AuditEventType::LoginFailure => "LOGIN_FAILURE",
            AuditEventType::OtpGenerated => "OTP_GENERATED",
            AuditEventType::OtpCopied => "OTP_COPIED",
            AuditEventType::SecretAccessed => "SECRET_ACCESSED",
            AuditEventType::SessionRevoked => "SESSION_REVOKED",
            AuditEventType::PermissionChanged => "PERMISSION_CHANGED",
            AuditEventType::ShiftAssignmentChanged => "SHIFT_ASSIGNMENT_CHANGED",
            AuditEventType::SuspiciousActivityDetected => "SUSPICIOUS_ACTIVITY_DETECTED",
            AuditEventType::FailedAccessAttempt => "FAILED_ACCESS_ATTEMPT",
            AuditEventType::ClipboardCleared => "CLIPBOARD_CLEARED",
            AuditEventType::ClipboardTimeout => "CLIPBOARD_TIMEOUT",
            AuditEventType::PlatformCreated => "PLATFORM_CREATED",
            AuditEventType::PlatformUpdated => "PLATFORM_UPDATED",
            AuditEventType::PlatformDeleted => "PLATFORM_DELETED",
        }
    }

    pub fn severity(&self) -> &'static str {
        match self {
            AuditEventType::LoginSuccess => "INFO",
            AuditEventType::LoginFailure => "WARN",
            AuditEventType::OtpGenerated => "INFO",
            AuditEventType::OtpCopied => "INFO",
            AuditEventType::SecretAccessed => "INFO",
            AuditEventType::SessionRevoked => "WARN",
            AuditEventType::PermissionChanged => "INFO",
            AuditEventType::ShiftAssignmentChanged => "INFO",
            AuditEventType::SuspiciousActivityDetected => "HIGH",
            AuditEventType::FailedAccessAttempt => "WARN",
            AuditEventType::ClipboardCleared => "INFO",
            AuditEventType::ClipboardTimeout => "INFO",
            AuditEventType::PlatformCreated => "INFO",
            AuditEventType::PlatformUpdated => "INFO",
            AuditEventType::PlatformDeleted => "WARN",
        }
    }
}

pub struct AuditEventBuilder {
    event_type: AuditEventType,
    user_id: Option<Uuid>,
    session_id: Option<Uuid>,
    platform_id: Option<Uuid>,
    workstation_id: Option<String>,
    metadata: serde_json::Value,
}

impl AuditEventBuilder {
    pub fn new(event_type: AuditEventType) -> Self {
        Self {
            event_type,
            user_id: None,
            session_id: None,
            platform_id: None,
            workstation_id: None,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_session_id(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_platform_id(mut self, platform_id: Uuid) -> Self {
        self.platform_id = Some(platform_id);
        self
    }

    pub fn with_workstation_id(mut self, workstation_id: &str) -> Self {
        self.workstation_id = Some(workstation_id.to_string());
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn build(self) -> AuditEvent {
        AuditEvent {
            id: Uuid::new_v4(),
            user_id: self.user_id,
            session_id: self.session_id,
            platform_id: self.platform_id,
            event_type: self.event_type.as_str().to_string(),
            severity: self.event_type.severity().to_string(),
            workstation_id: self.workstation_id,
            timestamp: Utc::now(),
            integrity_hash: String::new(), // Set by logger
            metadata: self.metadata,
        }
    }
}
