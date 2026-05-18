use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub platform_id: Option<Uuid>,
    pub event_type: String,
    pub severity: String,
    pub workstation_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub integrity_hash: String,
    pub metadata: serde_json::Value,
}
