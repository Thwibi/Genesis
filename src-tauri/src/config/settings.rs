use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub database_path: PathBuf,
    pub master_key_path: PathBuf,
    pub clipboard_timeout_secs: u64,
    pub session_timeout_minutes: u64,
    pub max_login_attempts: u32,
    pub otp_period_seconds: u64,
    pub otp_digits: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            database_path: Self::default_data_dir().join("genesis_soc_auth.db"),
            master_key_path: Self::default_data_dir().join("master_key.bin"),
            clipboard_timeout_secs: 30,
            session_timeout_minutes: 480,
            max_login_attempts: 5,
            otp_period_seconds: 30,
            otp_digits: 6,
        }
    }
}

impl Settings {
    fn default_data_dir() -> PathBuf {
        dirs_next::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("GenesisSOC")
    }

    pub fn load() -> Result<Self, String> {
        Ok(Self::default())
    }
}

// Use dirs-next for cross-platform data directories
mod dirs_next {
    use std::path::PathBuf;

    #[cfg(target_os = "windows")]
    pub fn data_dir() -> Option<PathBuf> {
        std::env::var("APPDATA").ok().map(PathBuf::from)
    }

    #[cfg(target_os = "linux")]
    pub fn data_dir() -> Option<PathBuf> {
        std::env::var("XDG_DATA_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".local").join("share"))
            })
    }

    #[cfg(target_os = "macos")]
    pub fn data_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
    }
}
