#[cfg(test)]
mod tests {
    use genesis_soc_auth::security::password::PasswordHasherService;

    use genesis_soc_auth::audit::integrity::HashChain;
    use uuid::Uuid;
    use chrono::Utc;
    use genesis_soc_auth::domain::entities::audit_event::AuditEvent;



    #[test]
    fn test_password_hashing_and_verification() {
        let password = "TestPassword123!";
        let hash = PasswordHasherService::hash(password).expect("Hashing should succeed");
        assert!(PasswordHasherService::verify(password, &hash).expect("Verification should succeed"));
        assert!(!PasswordHasherService::verify("WrongPassword", &hash).expect("Verification should succeed"));
    }

    #[test]
    fn test_different_passwords_produce_different_hashes() {
        let hash1 = PasswordHasherService::hash("password1").unwrap();
        let hash2 = PasswordHasherService::hash("password1").unwrap();
        assert_ne!(hash1, hash2, "Argon2id should use random salts");
    }

    #[test]
    fn test_hash_chain_integrity() {
        let event1 = AuditEvent {
            id: Uuid::new_v4(),
            user_id: None,
            session_id: None,
            platform_id: None,
            event_type: "LOGIN_SUCCESS".to_string(),
            severity: "INFO".to_string(),
            workstation_id: None,
            timestamp: Utc::now(),
            integrity_hash: String::new(),
            metadata: serde_json::json!({}),
        };

        let h1 = HashChain::compute_hash(&event1, None);
        assert_eq!(h1.len(), 64);

        let mut event2 = event1.clone();
        event2.id = Uuid::new_v4();
        event2.event_type = "OTP_COPIED".to_string();
        let h2 = HashChain::compute_hash(&event2, Some(&h1));
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_session_id_is_valid_uuid() {
        let id = Uuid::new_v4();
        let parsed = Uuid::parse_str(&id.to_string());
        assert!(parsed.is_ok());
        assert_eq!(parsed.unwrap(), id);
    }
}
