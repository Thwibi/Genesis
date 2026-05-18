use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub workstation_id: String,
    pub hostname: String,
    pub operating_system: String,
    pub login_timestamp: DateTime<Utc>,
    pub last_activity_timestamp: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub integrity_state: IntegrityState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IntegrityState {
    Intact,
    Compromised,
    Suspended,
}

impl IntegrityState {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntegrityState::Intact => "INTACT",
            IntegrityState::Compromised => "COMPROMISED",
            IntegrityState::Suspended => "SUSPENDED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "INTACT" => Some(IntegrityState::Intact),
            "COMPROMISED" => Some(IntegrityState::Compromised),
            "SUSPENDED" => Some(IntegrityState::Suspended),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkstationInfo {
    pub workstation_id: String,
    pub hostname: String,
    pub operating_system: String,
}
