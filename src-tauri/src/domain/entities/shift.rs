use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shift {
    pub id: Uuid,
    pub shift_name: String,
    pub shift_start: DateTime<Utc>,
    pub shift_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftAssignment {
    pub id: Uuid,
    pub shift_id: Uuid,
    pub user_id: Uuid,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShiftType {
    pub name: &'static str,
    pub start_hour: u32,
    pub end_hour: u32,
}

impl ShiftType {
    pub const MORNING: ShiftType = ShiftType { name: "Morning Shift", start_hour: 6, end_hour: 14 };
    pub const EVENING: ShiftType = ShiftType { name: "Evening Shift", start_hour: 14, end_hour: 22 };
    pub const NIGHT: ShiftType = ShiftType { name: "Night Shift", start_hour: 22, end_hour: 6 };
    pub const INCIDENT_RESPONSE: ShiftType = ShiftType { name: "Incident Response Shift", start_hour: 0, end_hour: 24 };

    pub fn all() -> Vec<ShiftType> {
        vec![Self::MORNING, Self::EVENING, Self::NIGHT, Self::INCIDENT_RESPONSE]
    }
}
