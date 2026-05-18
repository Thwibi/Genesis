use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::commands::CommandError;
use crate::persistence::repositories::audit_repo::AuditRepo;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct AuditEventDto {
    pub id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub platform_id: Option<String>,
    pub event_type: String,
    pub severity: String,
    pub workstation_id: Option<String>,
    pub timestamp: String,
    pub integrity_hash: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct AuditQueryParams {
    pub event_type: Option<String>,
    pub user_id: Option<String>,
    pub severity: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[tauri::command]
pub async fn cmd_query_audit_logs(
    state: State<'_, AppState>,
    params: AuditQueryParams,
) -> Result<Vec<AuditEventDto>, CommandError> {
    let conn = state.db_pool.connection().await
        .map_err(|e| CommandError {
            code: "DATABASE_ERROR".to_string(),
            message: e.to_string(),
        })?;

    let user_id = params.user_id
        .and_then(|s| Uuid::parse_str(&s).ok());

    let events = AuditRepo::query(
        &*conn,
        params.event_type.as_deref(),
        user_id,
        params.severity.as_deref(),
        params.limit.unwrap_or(100),
        params.offset.unwrap_or(0),
    ).map_err(|e| CommandError {
        code: "QUERY_FAILED".to_string(),
        message: e,
    })?;

    Ok(events.into_iter().map(|e| AuditEventDto {
        id: e.id.to_string(),
        user_id: e.user_id.map(|u| u.to_string()),
        session_id: e.session_id.map(|s| s.to_string()),
        platform_id: e.platform_id.map(|p| p.to_string()),
        event_type: e.event_type,
        severity: e.severity,
        workstation_id: e.workstation_id,
        timestamp: e.timestamp.to_rfc3339(),
        integrity_hash: e.integrity_hash,
        metadata: e.metadata,
    }).collect())
}

#[tauri::command]
pub async fn cmd_get_audit_event(
    state: State<'_, AppState>,
    event_id: String,
) -> Result<AuditEventDto, CommandError> {
    let eid = Uuid::parse_str(&event_id).map_err(|e| CommandError {
        code: "INVALID_EVENT_ID".to_string(),
        message: e.to_string(),
    })?;

    let conn = state.db_pool.connection().await
        .map_err(|e| CommandError {
            code: "DATABASE_ERROR".to_string(),
            message: e.to_string(),
        })?;

    let event = AuditRepo::find_by_id(&*conn, eid)
        .map_err(|e| CommandError {
            code: "EVENT_NOT_FOUND".to_string(),
            message: e,
        })?;

    Ok(AuditEventDto {
        id: event.id.to_string(),
        user_id: event.user_id.map(|u| u.to_string()),
        session_id: event.session_id.map(|s| s.to_string()),
        platform_id: event.platform_id.map(|p| p.to_string()),
        event_type: event.event_type,
        severity: event.severity,
        workstation_id: event.workstation_id,
        timestamp: event.timestamp.to_rfc3339(),
        integrity_hash: event.integrity_hash,
        metadata: event.metadata,
    })
}
