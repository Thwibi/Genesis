use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

use crate::commands::CommandError;
use crate::domain::entities::shift::Shift;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct ShiftDto {
    pub id: String,
    pub shift_name: String,
    pub shift_start: String,
    pub shift_end: String,
    pub created_at: String,
}

impl From<Shift> for ShiftDto {
    fn from(s: Shift) -> Self {
        Self {
            id: s.id.to_string(),
            shift_name: s.shift_name,
            shift_start: s.shift_start.to_rfc3339(),
            shift_end: s.shift_end.to_rfc3339(),
            created_at: s.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateShiftRequest {
    pub shift_name: String,
    pub shift_start: String,
    pub shift_end: String,
    pub created_by: String,
}

#[derive(Debug, Deserialize)]
pub struct AssignAnalystRequest {
    pub shift_id: String,
    pub analyst_id: String,
    pub assigned_by: String,
}

#[tauri::command]
pub async fn cmd_create_shift(
    state: State<'_, AppState>,
    request: CreateShiftRequest,
) -> Result<ShiftDto, CommandError> {
    let created_by = Uuid::parse_str(&request.created_by).map_err(|e| CommandError {
        code: "INVALID_USER_ID".to_string(),
        message: e.to_string(),
    })?;

    let start = DateTime::parse_from_rfc3339(&request.shift_start)
        .map_err(|e| CommandError {
            code: "INVALID_DATE".to_string(),
            message: e.to_string(),
        })?.with_timezone(&Utc);

    let end = DateTime::parse_from_rfc3339(&request.shift_end)
        .map_err(|e| CommandError {
            code: "INVALID_DATE".to_string(),
            message: e.to_string(),
        })?.with_timezone(&Utc);

    let svc = state.shift_service.read().await;
    let shift = svc.create_shift(&request.shift_name, start, end, created_by)
        .await
        .map_err(CommandError::from)?;

    Ok(ShiftDto::from(shift))
}

#[tauri::command]
pub async fn cmd_assign_analyst(
    state: State<'_, AppState>,
    request: AssignAnalystRequest,
) -> Result<(), CommandError> {
    use crate::domain::entities::shift::ShiftAssignment;
    let shift_id = Uuid::parse_str(&request.shift_id).map_err(|e| CommandError {
        code: "INVALID_SHIFT_ID".to_string(),
        message: e.to_string(),
    })?;
    let analyst_id = Uuid::parse_str(&request.analyst_id).map_err(|e| CommandError {
        code: "INVALID_USER_ID".to_string(),
        message: e.to_string(),
    })?;
    let assigned_by = Uuid::parse_str(&request.assigned_by).map_err(|e| CommandError {
        code: "INVALID_USER_ID".to_string(),
        message: e.to_string(),
    })?;

    let svc = state.shift_service.read().await;
    svc.assign_analyst(shift_id, analyst_id, assigned_by)
        .await
        .map_err(CommandError::from)
        .map(|_: ShiftAssignment| ())
}

#[tauri::command]
pub async fn cmd_list_shifts(
    state: State<'_, AppState>,
) -> Result<Vec<ShiftDto>, CommandError> {
    let svc = state.shift_service.read().await;
    let shifts = svc.list_shifts()
        .await
        .map_err(CommandError::from)?;

    Ok(shifts.into_iter().map(ShiftDto::from).collect())
}

#[tauri::command]
pub async fn cmd_get_active_shifts(
    state: State<'_, AppState>,
) -> Result<Vec<ShiftDto>, CommandError> {
    let svc = state.shift_service.read().await;
    let active = svc.get_active_shifts()
        .await
        .map_err(CommandError::from)?;

    Ok(active.into_iter().map(|(s, _)| ShiftDto::from(s)).collect())
}
