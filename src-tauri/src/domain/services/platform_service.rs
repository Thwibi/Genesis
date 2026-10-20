use std::sync::Arc;
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::domain::entities::platform::{CriticalityLevel, ProtectedPlatform};
use crate::domain::errors::DomainError;
use crate::audit::events::{AuditEventBuilder, AuditEventType};
use crate::audit::logger::AuditLogger;
use crate::persistence::db::DatabasePool;
use crate::persistence::repositories::platform_repo::PlatformRepo;

pub struct PlatformService {
    db_pool: Arc<DatabasePool>,
    audit_logger: Arc<AuditLogger>,
}

impl PlatformService {
    pub fn new(db_pool: Arc<DatabasePool>, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            db_pool,
            audit_logger,
        }
    }

    pub async fn list_platforms(&self) -> Result<Vec<ProtectedPlatform>, DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        PlatformRepo::find_all(&*conn)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    pub async fn create_platform(
        &self,
        name: &str,
        platform_type: &str,
        environment: &str,
        criticality: &str,
        assigned_teams: Vec<String>,
        session_id: Uuid,
    ) -> Result<ProtectedPlatform, DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let criticality_level = CriticalityLevel::from_str(criticality)
            .ok_or_else(|| DomainError::ValidationError(format!("Invalid criticality: {}", criticality)))?;

        let platform = ProtectedPlatform {
            id: Uuid::new_v4(),
            name: name.to_string(),
            platform_type: platform_type.to_string(),
            environment: environment.to_string(),
            criticality_level,
            assigned_teams,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        PlatformRepo::create(&*conn, &platform)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // conn dropped before audit write to avoid holding the pool connection
        drop(conn);

        let event = AuditEventBuilder::new(AuditEventType::PlatformCreated)
            .with_session_id(session_id)
            .with_platform_id(platform.id)
            .with_metadata(serde_json::json!({
                "name": platform.name,
                "platform_type": platform.platform_type,
                "environment": platform.environment,
                "criticality_level": format!("{:?}", platform.criticality_level),
            }))
            .build();

        self.audit_logger.write(event).await
            .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

        info!(target: "platform", platform_id = %platform.id, name = %name, "Platform created");

        Ok(platform)
    }

    pub async fn update_platform(
        &self,
        platform_id: Uuid,
        name: Option<String>,
        platform_type: Option<String>,
        environment: Option<String>,
        criticality: Option<String>,
        assigned_teams: Option<Vec<String>>,
        session_id: Uuid,
    ) -> Result<ProtectedPlatform, DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let mut platform = PlatformRepo::find_by_id(&*conn, platform_id)
            .map_err(|_| DomainError::PlatformNotFound(platform_id.to_string()))?;

        if let Some(name) = name { platform.name = name; }
        if let Some(pt) = platform_type { platform.platform_type = pt; }
        if let Some(env) = environment { platform.environment = env; }
        if let Some(crit) = criticality {
            platform.criticality_level = CriticalityLevel::from_str(&crit)
                .ok_or_else(|| DomainError::ValidationError(format!("Invalid criticality: {}", crit)))?;
        }
        if let Some(teams) = assigned_teams { platform.assigned_teams = teams; }

        platform.updated_at = Utc::now();

        PlatformRepo::update(&*conn, &platform)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        drop(conn);

        let event = AuditEventBuilder::new(AuditEventType::PlatformUpdated)
            .with_session_id(session_id)
            .with_platform_id(platform.id)
            .with_metadata(serde_json::json!({
                "name": platform.name,
                "platform_type": platform.platform_type,
                "environment": platform.environment,
                "criticality_level": format!("{:?}", platform.criticality_level),
            }))
            .build();

        self.audit_logger.write(event).await
            .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

        info!(target: "platform", platform_id = %platform_id, "Platform updated");

        Ok(platform)
    }

    pub async fn delete_platform(&self, platform_id: Uuid, session_id: Uuid) -> Result<(), DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Capture identity before deletion for audit metadata.
        let platform_name = PlatformRepo::find_by_id(&*conn, platform_id)
            .map(|p| p.name)
            .unwrap_or_else(|_| platform_id.to_string());

        PlatformRepo::delete(&*conn, platform_id)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        drop(conn);

        let event = AuditEventBuilder::new(AuditEventType::PlatformDeleted)
            .with_session_id(session_id)
            .with_platform_id(platform_id)
            .with_metadata(serde_json::json!({
                "name": platform_name,
            }))
            .build();

        self.audit_logger.write(event).await
            .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

        info!(target: "platform", platform_id = %platform_id, "Platform deleted");

        Ok(())
    }
}
