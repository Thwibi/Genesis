pub mod auth;
pub mod platforms;
pub mod otp;
pub mod secrets;
pub mod audit;
pub mod shifts;
pub mod monitoring;
pub mod setup;

use serde::Serialize;

/// Standard error type for all Tauri commands.
/// Maps domain errors to serializable command errors.
#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}
