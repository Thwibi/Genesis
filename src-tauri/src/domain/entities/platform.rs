use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedPlatform {
    pub id: Uuid,
    pub name: String,
    pub platform_type: String,
    pub environment: String,
    pub criticality_level: CriticalityLevel,
    pub assigned_teams: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CriticalityLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl CriticalityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            CriticalityLevel::Low => "LOW",
            CriticalityLevel::Medium => "MEDIUM",
            CriticalityLevel::High => "HIGH",
            CriticalityLevel::Critical => "CRITICAL",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "LOW" => Some(CriticalityLevel::Low),
            "MEDIUM" => Some(CriticalityLevel::Medium),
            "HIGH" => Some(CriticalityLevel::High),
            "CRITICAL" => Some(CriticalityLevel::Critical),
            _ => None,
        }
    }
}
