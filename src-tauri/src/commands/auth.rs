use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::commands::CommandError;
use crate::domain::entities::session::{Session, WorkstationInfo};
use crate::domain::errors::DomainError;
use crate::session::workstation;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct SessionDto {
    pub id: String,
    pub user_id: String,
    pub workstation_id: String,
    pub hostname: String,
    pub operating_system: String,
    pub login_timestamp: DateTime<Utc>,
    pub last_activity_timestamp: DateTime<Utc>,
    pub integrity_state: String,
}

impl From<Session> for SessionDto {
    fn from(s: Session) -> Self {
        Self {
            id: s.id.to_string(),
            user_id: s.user_id.to_string(),
            workstation_id: s.workstation_id,
            hostname: s.hostname,
            operating_system: s.operating_system,
            login_timestamp: s.login_timestamp,
            last_activity_timestamp: s.last_activity_timestamp,
            integrity_state: s.integrity_state.as_str().to_string(),
        }
    }
}

#[tauri::command]
pub async fn cmd_login(
    state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<SessionDto, CommandError> {
    let ws = workstation::collect().map_err(|e| CommandError {
        code: "WORKSTATION_ERROR".to_string(),
        message: e.to_string(),
    })?;

    let auth = state.auth_service.read().await;
    let session = auth.login(&username, &password, ws)
        .await
        .map_err(CommandError::from)?;

    Ok(SessionDto::from(session))
}

#[tauri::command]
pub async fn cmd_logout(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), CommandError> {
    let sid = Uuid::parse_str(&session_id).map_err(|e| CommandError {
        code: "INVALID_SESSION_ID".to_string(),
        message: e.to_string(),
    })?;

    let auth = state.auth_service.read().await;
    auth.logout(sid)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn cmd_revoke_session(
    state: State<'_, AppState>,
    session_id: String,
    revoked_by: String,
) -> Result<(), CommandError> {
    let sid = Uuid::parse_str(&session_id).map_err(|e| CommandError {
        code: "INVALID_SESSION_ID".to_string(),
        message: e.to_string(),
    })?;
    let rb = Uuid::parse_str(&revoked_by).map_err(|e| CommandError {
        code: "INVALID_USER_ID".to_string(),
        message: e.to_string(),
    })?;

    let auth = state.auth_service.read().await;
    auth.revoke_session(sid, rb)
        .await
        .map_err(CommandError::from)
}

#[tauri::command]
pub async fn cmd_validate_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<SessionDto, CommandError> {
    let sid = Uuid::parse_str(&session_id).map_err(|e| CommandError {
        code: "INVALID_SESSION_ID".to_string(),
        message: e.to_string(),
    })?;

    let auth = state.auth_service.read().await;
    let session = auth.validate_session(sid)
        .await
        .map_err(CommandError::from)?;

    Ok(SessionDto::from(session))
}
