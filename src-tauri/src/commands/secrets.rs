use base64::Engine as _;
use serde::Deserialize;
use tauri::State;
use uuid::Uuid;

use crate::commands::CommandError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct StoreSecretRequest {
    pub platform_id: String,
    pub raw_seed_b64: String,
    pub session_id: String,
}

#[tauri::command]
pub async fn cmd_store_secret(
    state: State<'_, AppState>,
    request: StoreSecretRequest,
) -> Result<String, CommandError> {
    let pid = Uuid::parse_str(&request.platform_id).map_err(|e| CommandError {
        code: "INVALID_PLATFORM_ID".to_string(),
        message: e.to_string(),
    })?;
    let sid = Uuid::parse_str(&request.session_id).map_err(|e| CommandError {
        code: "INVALID_SESSION_ID".to_string(),
        message: e.to_string(),
    })?;

    let raw_seed = base64::engine::general_purpose::STANDARD
        .decode(&request.raw_seed_b64)
        .map_err(|e| CommandError {
        code: "INVALID_BASE64".to_string(),
        message: e.to_string(),
    })?;

    let svc = state.secret_service.read().await;
    let secret_id = svc.store_secret(pid, &raw_seed, sid)
        .await
        .map_err(CommandError::from)?;

    Ok(secret_id.to_string())
}

#[derive(Debug, Deserialize)]
pub struct RotateSecretRequest {
    pub platform_id: String,
    pub new_seed_b64: String,
    pub session_id: String,
}

#[tauri::command]
pub async fn cmd_rotate_secret(
    state: State<'_, AppState>,
    request: RotateSecretRequest,
) -> Result<(), CommandError> {
    let pid = Uuid::parse_str(&request.platform_id).map_err(|e| CommandError {
        code: "INVALID_PLATFORM_ID".to_string(),
        message: e.to_string(),
    })?;
    let sid = Uuid::parse_str(&request.session_id).map_err(|e| CommandError {
        code: "INVALID_SESSION_ID".to_string(),
        message: e.to_string(),
    })?;

    let new_seed = base64::engine::general_purpose::STANDARD
        .decode(&request.new_seed_b64)
        .map_err(|e| CommandError {
        code: "INVALID_BASE64".to_string(),
        message: e.to_string(),
    })?;

    let svc = state.secret_service.read().await;
    svc.rotate_secret(pid, &new_seed, sid)
        .await
        .map_err(CommandError::from)
}

#[derive(Debug, Deserialize)]
pub struct ArchiveSecretRequest {
    pub platform_id: String,
    pub session_id: String,
}

#[tauri::command]
pub async fn cmd_archive_secret(
    state: State<'_, AppState>,
    request: ArchiveSecretRequest,
) -> Result<(), CommandError> {
    let pid = Uuid::parse_str(&request.platform_id).map_err(|e| CommandError {
        code: "INVALID_PLATFORM_ID".to_string(),
        message: e.to_string(),
    })?;
    let sid = Uuid::parse_str(&request.session_id).map_err(|e| CommandError {
        code: "INVALID_SESSION_ID".to_string(),
        message: e.to_string(),
    })?;

    let svc = state.secret_service.read().await;
    svc.archive_secret(pid, sid)
        .await
        .map_err(CommandError::from)
}
