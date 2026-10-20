use std::sync::Arc;
use chrono::{DateTime, Timelike, Utc};
use serde_json::json;
use tracing::warn;
use uuid::Uuid;

use crate::domain::errors::DomainError;
use crate::audit::events::{AuditEventBuilder, AuditEventType};
use crate::audit::logger::AuditLogger;
use crate::persistence::db::DatabasePool;
use crate::persistence::repositories::audit_repo::AuditRepo;
use crate::persistence::repositories::session_repo::SessionRepo;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityAlert {
    pub id: Uuid,
    pub event_type: String,
    pub severity: AlertSeverity,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub acknowledged: bool,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AlertSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertSeverity::Low => "LOW",
            AlertSeverity::Medium => "MEDIUM",
            AlertSeverity::High => "HIGH",
            AlertSeverity::Critical => "CRITICAL",
        }
    }
}

pub struct MonitoringService {
    db_pool: Arc<DatabasePool>,
    audit_logger: Arc<AuditLogger>,
    alerts: Arc<tokio::sync::RwLock<Vec<SecurityAlert>>>,
}

impl MonitoringService {
    pub fn new(db_pool: Arc<DatabasePool>, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            db_pool,
            audit_logger,
            alerts: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    pub async fn check_excessive_otp_copying(
        &self,
        user_id: Uuid,
        threshold: usize,
        window_minutes: i64,
    ) -> Result<(), DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let since = Utc::now() - chrono::Duration::minutes(window_minutes);
        let count = AuditRepo::count_events_by_user_since(&*conn, user_id, "OTP_COPIED", since)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if count > threshold as i64 {
            warn!(target: "monitoring", user_id = %user_id, count = %count, threshold = %threshold, "Excessive OTP copying detected");

            let event = AuditEventBuilder::new(AuditEventType::SuspiciousActivityDetected)
                .with_user_id(user_id)
                .with_metadata(json!({
                    "rule": "excessive_otp_copying",
                    "count": count,
                    "threshold": threshold,
                    "window_minutes": window_minutes,
                }))
                .build();

            self.audit_logger.write(event).await
                .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

            let alert = SecurityAlert {
                id: Uuid::new_v4(),
                event_type: "excessive_otp_copying".to_string(),
                severity: AlertSeverity::Medium,
                description: format!("User copied OTP {} times in {} minutes (threshold: {})", count, window_minutes, threshold),
                detected_at: Utc::now(),
                acknowledged: false,
                metadata: json!({"user_id": user_id.to_string(), "count": count}),
            };

            self.alerts.write().await.push(alert);
        }

        Ok(())
    }

    pub async fn check_off_shift_access(
        &self,
        user_id: Uuid,
        session: &crate::domain::entities::session::Session,
    ) -> Result<(), DomainError> {
        let current_hour = Utc::now().hour();
        // Standard shift hours: 6-22 (morning + evening), overnight is night shift
        let is_off_hours = current_hour >= 22 || current_hour < 6;

        if is_off_hours {
            warn!(target: "monitoring", user_id = %user_id, "Off-shift access detected");

            let event = AuditEventBuilder::new(AuditEventType::SuspiciousActivityDetected)
                .with_user_id(user_id)
                .with_session_id(session.id)
                .with_workstation_id(&session.workstation_id)
                .with_metadata(json!({
                    "rule": "off_shift_access",
                    "current_hour": current_hour,
                    "hostname": session.hostname,
                }))
                .build();

            self.audit_logger.write(event).await
                .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

            let alert = SecurityAlert {
                id: Uuid::new_v4(),
                event_type: "off_shift_access".to_string(),
                severity: AlertSeverity::Low,
                description: format!("User accessed system during off-shift hours (hour: {})", current_hour),
                detected_at: Utc::now(),
                acknowledged: false,
                metadata: json!({"user_id": user_id.to_string(), "hour": current_hour}),
            };

            self.alerts.write().await.push(alert);
        }

        Ok(())
    }

    pub async fn check_repeated_auth_failures(
        &self,
        username: &str,
        attempt_count: u32,
    ) -> Result<(), DomainError> {
        if attempt_count >= 3 {
            warn!(target: "monitoring", username = %username, attempts = %attempt_count, "Repeated authentication failures");

            let event = AuditEventBuilder::new(AuditEventType::SuspiciousActivityDetected)
                .with_metadata(json!({
                    "rule": "repeated_auth_failures",
                    "username": username,
                    "attempts": attempt_count,
                }))
                .build();

            self.audit_logger.write(event).await
                .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

            let severity = if attempt_count >= 5 { AlertSeverity::High } else { AlertSeverity::Medium };

            let alert = SecurityAlert {
                id: Uuid::new_v4(),
                event_type: "repeated_auth_failures".to_string(),
                severity,
                description: format!("User '{}' had {} consecutive failed login attempts", username, attempt_count),
                detected_at: Utc::now(),
                acknowledged: false,
                metadata: json!({"username": username, "attempts": attempt_count}),
            };

            self.alerts.write().await.push(alert);
        }

        Ok(())
    }

    pub async fn get_alerts(&self) -> Vec<SecurityAlert> {
        self.alerts.read().await.clone()
    }

    pub async fn acknowledge_alert(&self, alert_id: Uuid) -> Result<(), DomainError> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            Ok(())
        } else {
            Err(DomainError::InternalError("Alert not found".to_string()))
        }
    }

    pub async fn check_unauthorized_workstation(
        &self,
        user_id: Uuid,
        workstation_id: &str,
    ) -> Result<(), DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Check if user has active sessions from other workstations
        let active_sessions = SessionRepo::find_active_by_user(&*conn, user_id)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        for session in &active_sessions {
            if session.workstation_id != workstation_id {
                warn!(target: "monitoring", user_id = %user_id, "Simultaneous access from different workstations");

                let event = AuditEventBuilder::new(AuditEventType::SuspiciousActivityDetected)
                    .with_user_id(user_id)
                    .with_session_id(session.id)
                    .with_workstation_id(workstation_id)
                    .with_metadata(json!({
                        "rule": "unauthorized_workstation",
                        "existing_workstation": session.workstation_id,
                        "new_workstation": workstation_id,
                    }))
                    .build();

                self.audit_logger.write(event).await
                    .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

                let alert = SecurityAlert {
                    id: Uuid::new_v4(),
                    event_type: "unauthorized_workstation".to_string(),
                    severity: AlertSeverity::High,
                    description: format!("User has concurrent sessions from different workstations"),
                    detected_at: Utc::now(),
                    acknowledged: false,
                    metadata: json!({"user_id": user_id.to_string()}),
                };

                self.alerts.write().await.push(alert);
            }
        }

        Ok(())
    }

    pub async fn check_abnormal_access_frequency(
        &self,
        user_id: Uuid,
        platform_id: Uuid,
    ) -> Result<(), DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let since = Utc::now() - chrono::Duration::minutes(5);
        let count = AuditRepo::count_platform_access_since(&*conn, user_id, platform_id, since)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        if count > 10 {
            warn!(target: "monitoring", user_id = %user_id, platform_id = %platform_id, count = %count, "Abnormal platform access frequency");

            let event = AuditEventBuilder::new(AuditEventType::SuspiciousActivityDetected)
                .with_user_id(user_id)
                .with_platform_id(platform_id)
                .with_metadata(json!({
                    "rule": "abnormal_access_frequency",
                    "access_count": count,
                    "window_minutes": 5,
                }))
                .build();

            self.audit_logger.write(event).await
                .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

            let alert = SecurityAlert {
                id: Uuid::new_v4(),
                event_type: "abnormal_access_frequency".to_string(),
                severity: AlertSeverity::Medium,
                description: format!("User accessed platform {} times in 5 minutes", count),
                detected_at: Utc::now(),
                acknowledged: false,
                metadata: json!({"user_id": user_id.to_string(), "platform_id": platform_id.to_string(), "count": count}),
            };

            self.alerts.write().await.push(alert);
        }

        Ok(())
    }
}
