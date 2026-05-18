use sha2::{Digest, Sha256};
use crate::domain::entities::audit_event::AuditEvent;

pub struct HashChain;

impl HashChain {
    /// Compute the SHA-256 integrity hash for an audit event.
    ///
    /// The hash covers:
    /// - All event fields serialized in a canonical format
    /// - The previous row's integrity hash (if any), creating a chain
    ///
    /// Format: SHA256(id || timestamp || event_type || user_id || session_id || platform_id || workstation_id || severity || metadata_json || previous_hash)
    pub fn compute_hash(event: &AuditEvent, previous_hash: Option<&str>) -> String {
        let mut hasher = Sha256::new();

        hasher.update(event.id.to_string().as_bytes());
        hasher.update(event.timestamp.to_rfc3339().as_bytes());
        hasher.update(event.event_type.as_bytes());
        hasher.update(event.user_id.map(|u| u.to_string()).unwrap_or_default().as_bytes());
        hasher.update(event.session_id.map(|u| u.to_string()).unwrap_or_default().as_bytes());
        hasher.update(event.platform_id.map(|u| u.to_string()).unwrap_or_default().as_bytes());
        hasher.update(event.workstation_id.as_deref().unwrap_or("").as_bytes());
        hasher.update(event.severity.as_bytes());
        hasher.update(event.metadata.to_string().as_bytes());

        if let Some(prev) = previous_hash {
            hasher.update(prev.as_bytes());
        }

        hex::encode(hasher.finalize())
    }

    /// Verify the integrity chain from a list of events.
    /// Returns Ok(()) if the chain is valid, or an error describing the first break.
    pub fn verify_chain(events: &[AuditEvent]) -> Result<(), String> {
        if events.is_empty() {
            return Ok(());
        }

        // Sort by timestamp to reconstruct chain order
        let mut sorted = events.to_vec();
        sorted.sort_by_key(|e| e.timestamp);

        let mut previous_hash: Option<String> = None;

        for event in &sorted {
            let computed = Self::compute_hash(event, previous_hash.as_deref());

            if event.integrity_hash != computed {
                return Err(format!(
                    "Integrity breach at event {}: hash mismatch. Expected {}, got {}",
                    event.id, computed, event.integrity_hash
                ));
            }

            previous_hash = Some(event.integrity_hash.clone());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_test_event() -> AuditEvent {
        AuditEvent {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            user_id: Some(Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()),
            session_id: None,
            platform_id: None,
            event_type: "LOGIN_SUCCESS".to_string(),
            severity: "INFO".to_string(),
            workstation_id: Some("ws-001".to_string()),
            timestamp: Utc::now(),
            integrity_hash: String::new(),
            metadata: serde_json::json!({"test": true}),
        }
    }

    #[test]
    fn test_hash_chain_consistency() {
        let event1 = make_test_event();
        let hash1 = HashChain::compute_hash(&event1, None);
        assert!(!hash1.is_empty());
        assert_eq!(hash1.len(), 64); // SHA-256 hex is 64 chars

        let hash1_again = HashChain::compute_hash(&event1, None);
        assert_eq!(hash1, hash1_again);
    }

    #[test]
    fn test_hash_chain_linking() {
        let event1 = make_test_event();
        let hash1 = HashChain::compute_hash(&event1, None);

        let mut event2 = make_test_event();
        event2.id = Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap();
        let hash2 = HashChain::compute_hash(&event2, Some(&hash1));

        assert_ne!(hash1, hash2);

        // Verify the chain
        let mut e1 = event1.clone();
        e1.integrity_hash = hash1.clone();
        let mut e2 = event2.clone();
        e2.integrity_hash = hash2.clone();

        assert!(HashChain::verify_chain(&[e1, e2]).is_ok());
    }

    #[test]
    fn test_hash_chain_tamper_detection() {
        let mut event1 = make_test_event();
        event1.integrity_hash = HashChain::compute_hash(&event1, None);

        let mut event2 = make_test_event();
        event2.id = Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap();
        event2.integrity_hash = HashChain::compute_hash(&event2, Some(&event1.integrity_hash));

        // Tamper with event2
        event2.event_type = "OTP_COPIED".to_string();

        assert!(HashChain::verify_chain(&[event1, event2]).is_err());
    }
}
