use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::commands::CommandError;
use crate::domain::entities::user::{AccountStatus, User, UserRole};
use crate::persistence::repositories::user_repo::UserRepo;
use crate::security::password::PasswordHasherService;
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct FirstRunStatus {
    pub is_first_run: bool,
}

#[tauri::command]
pub async fn cmd_check_users(
    state: State<'_, AppState>,
) -> Result<FirstRunStatus, CommandError> {
    let conn = state.db_pool.connection().await
        .map_err(|e| CommandError {
            code: "DB_ERROR".to_string(),
            message: e.to_string(),
        })?;
    let count = UserRepo::count(&*conn)
        .map_err(|e| CommandError {
            code: "DB_ERROR".to_string(),
            message: e,
        })?;
    Ok(FirstRunStatus { is_first_run: count == 0 })
}

#[derive(Debug, Serialize)]
pub struct SetupResult {
    pub user_id: String,
    pub username: String,
}

#[tauri::command]
pub async fn cmd_create_first_user(
    state: State<'_, AppState>,
    username: String,
    email: String,
    password: String,
) -> Result<SetupResult, CommandError> {
    let conn = state.db_pool.connection().await
        .map_err(|e| CommandError {
            code: "DB_ERROR".to_string(),
            message: e.to_string(),
        })?;

    let count = UserRepo::count(&*conn)
        .map_err(|e| CommandError {
            code: "DB_ERROR".to_string(),
            message: e,
        })?;

    if count > 0 {
        return Err(CommandError {
            code: "ALREADY_SETUP".to_string(),
            message: "System already has users. First-run setup is no longer available.".to_string(),
        });
    }

    if username.trim().is_empty() || password.trim().is_empty() {
        return Err(CommandError {
            code: "VALIDATION".to_string(),
            message: "Username and password are required.".to_string(),
        });
    }

    if email.trim().is_empty() || !email.contains('@') {
        return Err(CommandError {
            code: "VALIDATION".to_string(),
            message: "A valid email address is required.".to_string(),
        });
    }

    if password.len() < 8 {
        return Err(CommandError {
            code: "VALIDATION".to_string(),
            message: "Password must be at least 8 characters.".to_string(),
        });
    }

    let password_hash = PasswordHasherService::hash(&password)
        .map_err(|e| CommandError {
            code: "HASH_ERROR".to_string(),
            message: e.to_string(),
        })?;

    let user = User {
        id: Uuid::new_v4(),
        username: username.trim().to_string(),
        email: email.trim().to_string(),
        password_hash,
        role: UserRole::SecurityAdministrator,
        assigned_shift_id: None,
        account_status: AccountStatus::Active,
        created_at: chrono::Utc::now(),
        last_login_at: None,
    };

    UserRepo::create(&*conn, &user)
        .map_err(|e| CommandError {
            code: "CREATE_FAILED".to_string(),
            message: e,
        })?;

    Ok(SetupResult {
        user_id: user.id.to_string(),
        username: user.username,
    })
}
