use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::commands::CommandError;
use crate::domain::services::monitoring_service::SecurityAlert;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct SecurityAlertDto {
    pub id: String,
    pub event_type: String,
    pub severity: String,
    pub description: String,
    pub detected_at: String,
    pub acknowledged: bool,
    pub metadata: serde_json::Value,
}

impl From<SecurityAlert> for SecurityAlertDto {
    fn from(a: SecurityAlert) -> Self {
        Self {
            id: a.id.to_string(),
            event_type: a.event_type,
            severity: a.severity.as_str().to_string(),
            description: a.description,
            detected_at: a.detected_at.to_rfc3339(),
            acknowledged: a.acknowledged,
            metadata: a.metadata,
        }
    }
}

#[tauri::command]
pub async fn cmd_get_security_alerts(
    state: State<'_, AppState>,
) -> Result<Vec<SecurityAlertDto>, CommandError> {
    let svc = state.monitoring_service.read().await;
    let alerts = svc.get_alerts().await;
    Ok(alerts.into_iter().map(SecurityAlertDto::from).collect())
}

#[tauri::command]
pub async fn cmd_acknowledge_alert(
    state: State<'_, AppState>,
    alert_id: String,
) -> Result<(), CommandError> {
    let aid = Uuid::parse_str(&alert_id).map_err(|e| CommandError {
        code: "INVALID_ALERT_ID".to_string(),
        message: e.to_string(),
    })?;

    let svc = state.monitoring_service.read().await;
    svc.acknowledge_alert(aid)
        .await
        .map_err(CommandError::from)
}
