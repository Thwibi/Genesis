#[cfg(test)]
mod tests {
    use genesis_soc_auth::audit::events::{AuditEventBuilder, AuditEventType};
    use genesis_soc_auth::audit::integrity::HashChain;
    use uuid::Uuid;

    #[test]
    fn test_audit_event_builder_minimal() {
        let event = AuditEventBuilder::new(AuditEventType::LoginSuccess)
            .build();

        assert_eq!(event.event_type, "LOGIN_SUCCESS");
        assert_eq!(event.severity, "INFO");
        assert!(event.user_id.is_none());
    }

    #[test]
    fn test_audit_event_builder_full() {
        let user_id = Uuid::new_v4();
        let session_id = Uuid::new_v4();
        let platform_id = Uuid::new_v4();

        let event = AuditEventBuilder::new(AuditEventType::OtpCopied)
            .with_user_id(user_id)
            .with_session_id(session_id)
            .with_platform_id(platform_id)
            .with_workstation_id("WS-TEST-001")
            .with_metadata(serde_json::json!({
                "clipboard_timeout": 30,
                "otp_length": 6,
            }))
            .build();

        assert_eq!(event.event_type, "OTP_COPIED");
        assert_eq!(event.user_id, Some(user_id));
        assert_eq!(event.platform_id, Some(platform_id));
    }

    #[test]
    fn test_all_audit_event_types_have_str() {
        let types = vec![
            AuditEventType::LoginSuccess,
            AuditEventType::LoginFailure,
            AuditEventType::OtpGenerated,
            AuditEventType::OtpCopied,
            AuditEventType::SecretAccessed,
            AuditEventType::SessionRevoked,
            AuditEventType::PermissionChanged,
            AuditEventType::ShiftAssignmentChanged,
            AuditEventType::SuspiciousActivityDetected,
            AuditEventType::FailedAccessAttempt,
            AuditEventType::ClipboardCleared,
            AuditEventType::ClipboardTimeout,
        ];

        for t in types {
            let s = t.as_str();
            assert!(!s.is_empty(), "Event type should have a string representation");
            // Severity should not panic
            let _ = t.severity();
        }
    }

    #[test]
    fn test_hash_chain_verification_empty() {
        assert!(HashChain::verify_chain(&[]).is_ok());
    }
}
