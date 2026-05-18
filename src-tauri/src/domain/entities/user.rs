use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub assigned_shift_id: Option<Uuid>,
    pub account_status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserRole {
    SocAnalyst,
    SocSupervisor,
    SecurityAdministrator,
    SystemAdministrator,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::SocAnalyst => "SOC_ANALYST",
            UserRole::SocSupervisor => "SOC_SUPERVISOR",
            UserRole::SecurityAdministrator => "SECURITY_ADMINISTRATOR",
            UserRole::SystemAdministrator => "SYSTEM_ADMINISTRATOR",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "SOC_ANALYST" => Some(UserRole::SocAnalyst),
            "SOC_SUPERVISOR" => Some(UserRole::SocSupervisor),
            "SECURITY_ADMINISTRATOR" => Some(UserRole::SecurityAdministrator),
            "SYSTEM_ADMINISTRATOR" => Some(UserRole::SystemAdministrator),
            _ => None,
        }
    }

    pub fn can_access_secrets(&self) -> bool {
        matches!(self, UserRole::SocAnalyst | UserRole::SocSupervisor | UserRole::SecurityAdministrator)
    }

    pub fn can_manage_platforms(&self) -> bool {
        matches!(self, UserRole::SecurityAdministrator | UserRole::SystemAdministrator)
    }

    pub fn can_manage_shifts(&self) -> bool {
        matches!(self, UserRole::SocSupervisor | UserRole::SecurityAdministrator)
    }

    pub fn can_view_audit_logs(&self) -> bool {
        matches!(self, UserRole::SocSupervisor | UserRole::SecurityAdministrator | UserRole::SystemAdministrator)
    }

    pub fn can_manage_users(&self) -> bool {
        matches!(self, UserRole::SecurityAdministrator | UserRole::SystemAdministrator)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountStatus {
    Active,
    Locked,
    Disabled,
}

impl AccountStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccountStatus::Active => "ACTIVE",
            AccountStatus::Locked => "LOCKED",
            AccountStatus::Disabled => "DISABLED",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ACTIVE" => Some(AccountStatus::Active),
            "LOCKED" => Some(AccountStatus::Locked),
            "DISABLED" => Some(AccountStatus::Disabled),
            _ => None,
        }
    }
}
