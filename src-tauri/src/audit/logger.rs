use std::sync::Arc;
use chrono::Utc;
use tracing::{error, warn};

use crate::domain::entities::audit_event::AuditEvent;
use crate::audit::integrity::HashChain;
use crate::persistence::db::DatabasePool;
use crate::persistence::repositories::audit_repo::AuditRepo;

pub struct AuditLogger {
    db_pool: Arc<DatabasePool>,
}

impl AuditLogger {
    pub fn new(db_pool: Arc<DatabasePool>) -> Self {
        Self { db_pool }
    }

    /// Write an audit event to the append-only log.
    ///
    /// This computes the integrity hash chain before persisting.
    /// If the write fails, the calling operation MUST also fail.
    pub async fn write(&self, mut event: AuditEvent) -> Result<(), AuditLoggerError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| AuditLoggerError::DatabaseError(e.to_string()))?;

        // Compute integrity hash
        let previous_hash = AuditRepo::get_latest_integrity_hash(&conn)
            .map_err(|e| AuditLoggerError::DatabaseError(e.to_string()))?;

        event.integrity_hash = HashChain::compute_hash(&event, previous_hash.as_deref());

        // Append-only write
        AuditRepo::insert(&conn, &event)
            .map_err(|e| AuditLoggerError::WriteFailed(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum AuditLoggerError {
    WriteFailed(String),
    DatabaseError(String),
}

impl std::fmt::Display for AuditLoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditLoggerError::WriteFailed(msg) => write!(f, "Audit write failed: {}", msg),
            AuditLoggerError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for AuditLoggerError {}
