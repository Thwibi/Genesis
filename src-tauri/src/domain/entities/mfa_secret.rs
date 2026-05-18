use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaSecret {
    pub id: Uuid,
    pub platform_id: Uuid,
    pub encrypted_secret: Vec<u8>,
    pub algorithm: String,
    pub digits: u32,
    pub period: u64,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
}
