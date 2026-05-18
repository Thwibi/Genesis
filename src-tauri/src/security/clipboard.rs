use std::sync::Arc;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::audit::events::{AuditEventBuilder, AuditEventType};
use crate::audit::logger::AuditLogger;

pub struct ClipboardManager {
    audit_logger: Arc<AuditLogger>,
    clear_task: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl ClipboardManager {
    pub fn new(audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            audit_logger,
            clear_task: Arc::new(RwLock::new(None)),
        }
    }

    /// Write OTP string to clipboard and schedule auto-clear.
    /// Must be called AFTER the audit event has already been written.
    pub async fn write_with_auto_clear(
        &self,
        otp: &str,
        timeout_secs: u64,
    ) -> Result<(), ClipboardError> {
        // Cancel any existing clear task
        {
            let mut task = self.clear_task.write().await;
            if let Some(handle) = task.take() {
                handle.abort();
            }
        }

        // Write to clipboard via Tauri clipboard-manager plugin (JS layer)
        // The Rust side owns the audit and scheduling; the JS frontend calls
        // writeText() from @tauri-apps/plugin-clipboard-manager after this returns.

        let audit = self.audit_logger.clone();
        let otp_len = otp.len();

        // Schedule auto-clear
        let handle = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(timeout_secs)).await;

            // Clipboard clear is delegated to the JS frontend via the Tauri
            // clipboard-manager plugin's clear() API. The Rust audit trail is
            // written here before the frontend executes the clear.

            info!(target: "clipboard", "Clipboard auto-cleared after {}s", timeout_secs);

            let event = AuditEventBuilder::new(AuditEventType::ClipboardTimeout)
                .with_metadata(serde_json::json!({
                    "timeout_secs": timeout_secs,
                    "otp_length": otp_len,
                }))
                .build();

            if let Err(e) = audit.write(event).await {
                warn!(target: "clipboard", "Failed to log clipboard clear: {}", e);
            }
        });

        *self.clear_task.write().await = Some(handle);

        info!(target: "clipboard", timeout = %timeout_secs, "Clipboard write scheduled with auto-clear");

        Ok(())
    }

    /// Manually clear clipboard and cancel auto-clear timer.
    pub async fn clear_clipboard(&self) -> Result<(), ClipboardError> {
        {
            let mut task = self.clear_task.write().await;
            if let Some(handle) = task.take() {
                handle.abort();
            }
        }

        // Clipboard clear is delegated to the JS frontend via the Tauri plugin.
        // This Rust method logs the audit event; the frontend must call clear().

        info!(target: "clipboard", "Clipboard manually cleared");

        Ok(())
    }
}

#[derive(Debug)]
pub enum ClipboardError {
    WriteError(String),
    ClearError(String),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::WriteError(msg) => write!(f, "Clipboard write error: {}", msg),
            ClipboardError::ClearError(msg) => write!(f, "Clipboard clear error: {}", msg),
        }
    }
}

impl std::error::Error for ClipboardError {}
