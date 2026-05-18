use ring::digest::{digest, SHA256};
use tracing::warn;

use crate::domain::entities::session::WorkstationInfo;

/// Collect workstation metadata for session attribution.
pub fn collect() -> Result<WorkstationInfo, WorkstationError> {
    let hostname = get_hostname()?;
    let os = get_os_version();
    let workstation_id = get_workstation_id(&hostname);

    Ok(WorkstationInfo {
        workstation_id,
        hostname,
        operating_system: os,
    })
}

fn get_hostname() -> Result<String, WorkstationError> {
    #[cfg(target_os = "windows")]
    {
        // Uses COMPUTERNAME env var (set by Windows). For hardened deployments,
        // replace with GetComputerNameExW (requires unsafe) to get NetBIOS name
        // without a full FQDN.
        Ok(std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
            .unwrap_or_else(|_| {
                warn!(target: "workstation", "Could not determine hostname");
                "UNKNOWN".to_string()
            }))
    }

    #[cfg(target_os = "linux")]
    {
        std::env::var("HOSTNAME")
            .or_else(|_| {
                std::fs::read_to_string("/etc/hostname")
                    .map(|s| s.trim().to_string())
            })
            .map_err(|e| WorkstationError::HostnameError(e.to_string()))
    }

    #[cfg(target_os = "macos")]
    {
        std::env::var("HOSTNAME")
            .map_err(|e| WorkstationError::HostnameError(e.to_string()))
    }
}

fn get_os_version() -> String {
    #[cfg(target_os = "windows")]
    {
        // For accurate build number, use RtlGetVersion (requires unsafe).
        // The environment approach provides a reasonable approximation.
        let os = std::env::var("OS").unwrap_or_else(|_| "Windows_NT".to_string());
        if os == "Windows_NT" {
            "Windows 10+".to_string()
        } else {
            os
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Read /etc/os-release for distribution info in production deployments.
        std::fs::read_to_string("/etc/os-release")
            .ok()
            .and_then(|content| {
                content.lines()
                    .find(|l| l.starts_with("PRETTY_NAME="))
                    .and_then(|l| l.split('=').nth(1))
                    .map(|v| v.trim_matches('"').to_string())
            })
            .unwrap_or_else(|| "Linux".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        "macOS".to_string()
    }
}

fn get_workstation_id(hostname: &str) -> String {
    let hash = digest(&SHA256, hostname.as_bytes());
    let hex_hash = hex::encode(hash.as_ref());
    format!("WS-{}", &hex_hash[..16])
}

#[derive(Debug)]
pub enum WorkstationError {
    HostnameError(String),
}

impl std::fmt::Display for WorkstationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkstationError::HostnameError(msg) => write!(f, "Hostname error: {}", msg),
        }
    }
}

impl std::error::Error for WorkstationError {}
