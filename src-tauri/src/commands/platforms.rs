use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::commands::CommandError;
use crate::domain::entities::platform::ProtectedPlatform;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct PlatformDto {
    pub id: String,
    pub name: String,
    pub platform_type: String,
    pub environment: String,
    pub criticality_level: String,
    pub assigned_teams: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ProtectedPlatform> for PlatformDto {
    fn from(p: ProtectedPlatform) -> Self {
        Self {
            id: p.id.to_string(),
            name: p.name,
            platform_type: p.platform_type,
            environment: p.environment,
            criticality_level: p.criticality_level.as_str().to_string(),
            assigned_teams: p.assigned_teams,
            created_at: p.created_at.to_rfc3339(),
            updated_at: p.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreatePlatformRequest {
    pub name: String,
    pub platform_type: String,
    pub environment: String,
    pub criticality_level: String,
    pub assigned_teams: Vec<String>,
    pub session_id: String,
}

#[tauri::command]
pub async fn cmd_list_platforms(
    state: State<'_, AppState>,
) -> Result<Vec<PlatformDto>, CommandError> {
    let svc = state.platform_service.read().await;
    let platforms = svc.list_platforms()
        .await
        .map_err(CommandError::from)?;

    Ok(platforms.into_iter().map(PlatformDto::from).collect())
}

#[tauri::command]
pub async fn cmd_create_platform(
    state: State<'_, AppState>,
    request: CreatePlatformRequest,
) -> Result<PlatformDto, CommandError> {
    let sid = Uuid::parse_str(&request.session_id).map_err(|e| CommandError {
        code: "INVALID_SESSION_ID".to_string(),
        message: e.to_string(),
    })?;

    let svc = state.platform_service.read().await;
    let platform = svc.create_platform(
        &request.name,
        &request.platform_type,
        &request.environment,
        &request.criticality_level,
        request.assigned_teams,
        sid,
    )
    .await
    .map_err(CommandError::from)?;

    Ok(PlatformDto::from(platform))
}

#[tauri::command]
pub async fn cmd_update_platform(
    state: State<'_, AppState>,
    platform_id: String,
    name: Option<String>,
    platform_type: Option<String>,
    environment: Option<String>,
    criticality_level: Option<String>,
    assigned_teams: Option<Vec<String>>,
    session_id: String,
) -> Result<PlatformDto, CommandError> {
    let pid = Uuid::parse_str(&platform_id).map_err(|e| CommandError {
        code: "INVALID_PLATFORM_ID".to_string(),
        message: e.to_string(),
    })?;
    let sid = Uuid::parse_str(&session_id).map_err(|e| CommandError {
        code: "INVALID_SESSION_ID".to_string(),
        message: e.to_string(),
    })?;

    let svc = state.platform_service.read().await;
    let platform = svc.update_platform(pid, name, platform_type, environment, criticality_level, assigned_teams, sid)
        .await
        .map_err(CommandError::from)?;

    Ok(PlatformDto::from(platform))
}

#[tauri::command]
pub async fn cmd_delete_platform(
    state: State<'_, AppState>,
    platform_id: String,
    session_id: String,
) -> Result<(), CommandError> {
    let pid = Uuid::parse_str(&platform_id).map_err(|e| CommandError {
        code: "INVALID_PLATFORM_ID".to_string(),
        message: e.to_string(),
    })?;
    let sid = Uuid::parse_str(&session_id).map_err(|e| CommandError {
        code: "INVALID_SESSION_ID".to_string(),
        message: e.to_string(),
    })?;

    let svc = state.platform_service.read().await;
    svc.delete_platform(pid, sid)
        .await
        .map_err(CommandError::from)
}
