use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::commands::CommandError;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct OtpDto {
    pub otp: String,
    pub clipboard_timeout_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct GenerateOtpRequest {
    pub session_id: String,
    pub platform_id: String,
}

#[tauri::command]
pub async fn cmd_generate_otp(
    state: State<'_, AppState>,
    request: GenerateOtpRequest,
) -> Result<OtpDto, CommandError> {
    let sid = Uuid::parse_str(&request.session_id).map_err(|e| CommandError {
        code: "INVALID_SESSION_ID".to_string(),
        message: e.to_string(),
    })?;
    let pid = Uuid::parse_str(&request.platform_id).map_err(|e| CommandError {
        code: "INVALID_PLATFORM_ID".to_string(),
        message: e.to_string(),
    })?;

    let svc = state.otp_service.read().await;
    let otp = svc.generate_otp(sid, pid)
        .await
        .map_err(CommandError::from)?;

    Ok(OtpDto {
        otp,
        clipboard_timeout_secs: 30,
    })
}

#[tauri::command]
pub async fn cmd_copy_otp(
    state: State<'_, AppState>,
    session_id: String,
    platform_id: String,
    otp: String,
) -> Result<(), CommandError> {
    let sid = Uuid::parse_str(&session_id).map_err(|e| CommandError {
        code: "INVALID_SESSION_ID".to_string(),
        message: e.to_string(),
    })?;
    let pid = Uuid::parse_str(&platform_id).map_err(|e| CommandError {
        code: "INVALID_PLATFORM_ID".to_string(),
        message: e.to_string(),
    })?;

    // SECURITY: Audit event is written before clipboard operation
    let svc = state.otp_service.read().await;
    svc.copy_otp_to_clipboard(sid, pid, otp)
        .await
        .map_err(CommandError::from)?;

    // Clipboard write handled by the frontend using Tauri clipboard plugin
    Ok(())
}
