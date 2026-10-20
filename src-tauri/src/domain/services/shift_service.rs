use std::sync::Arc;
use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::domain::entities::shift::{Shift, ShiftAssignment};
use crate::domain::errors::DomainError;
use crate::audit::events::{AuditEventBuilder, AuditEventType};
use crate::audit::logger::AuditLogger;
use crate::persistence::db::DatabasePool;
use crate::persistence::repositories::shift_repo::ShiftRepo;

pub struct ShiftService {
    db_pool: Arc<DatabasePool>,
    audit_logger: Arc<AuditLogger>,
}

impl ShiftService {
    pub fn new(db_pool: Arc<DatabasePool>, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            db_pool,
            audit_logger,
        }
    }

    pub async fn create_shift(
        &self,
        name: &str,
        start: chrono::DateTime<Utc>,
        end: chrono::DateTime<Utc>,
        _created_by: Uuid,
    ) -> Result<Shift, DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let shift = Shift {
            id: Uuid::new_v4(),
            shift_name: name.to_string(),
            shift_start: start,
            shift_end: end,
            created_at: Utc::now(),
        };

        ShiftRepo::create(&*conn, &shift)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        info!(target: "shift", shift_id = %shift.id, name = %name, "Shift created");

        Ok(shift)
    }

    pub async fn assign_analyst(
        &self,
        shift_id: Uuid,
        analyst_id: Uuid,
        assigned_by: Uuid,
    ) -> Result<ShiftAssignment, DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let assignment = ShiftAssignment {
            id: Uuid::new_v4(),
            shift_id,
            user_id: analyst_id,
            assigned_at: Utc::now(),
        };

        ShiftRepo::assign_analyst(&*conn, &assignment)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Audit shift assignment change
        let event = AuditEventBuilder::new(AuditEventType::ShiftAssignmentChanged)
            .with_user_id(analyst_id)
            .with_metadata(serde_json::json!({
                "shift_id": shift_id.to_string(),
                "assigned_by": assigned_by.to_string(),
                "action": "assign",
            }))
            .build();

        self.audit_logger.write(event).await
            .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

        info!(target: "shift", shift_id = %shift_id, analyst_id = %analyst_id, "Analyst assigned to shift");

        Ok(assignment)
    }

    pub async fn list_shifts(&self) -> Result<Vec<Shift>, DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        ShiftRepo::find_all(&*conn)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))
    }

    pub async fn get_active_shifts(&self) -> Result<Vec<(Shift, Vec<ShiftAssignment>)>, DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let now = Utc::now();
        let all_shifts = ShiftRepo::find_all(&*conn)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let active_shifts: Vec<Shift> = all_shifts
            .into_iter()
            .filter(|s| s.shift_start <= now && s.shift_end >= now)
            .collect();

        let mut result = Vec::new();
        for shift in active_shifts {
            let assignments = ShiftRepo::find_assignments_by_shift(&*conn, shift.id)
                .map_err(|e| DomainError::DatabaseError(e.to_string()))?;
            result.push((shift, assignments));
        }

        Ok(result)
    }
}
