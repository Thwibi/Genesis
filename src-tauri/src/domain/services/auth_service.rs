use std::sync::Arc;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::domain::entities::session::{IntegrityState, Session, WorkstationInfo};
use crate::domain::entities::user::{AccountStatus, User, UserRole};
use crate::domain::errors::DomainError;
use crate::audit::events::{AuditEventBuilder, AuditEventType};
use crate::audit::logger::AuditLogger;
use crate::persistence::db::DatabasePool;
use crate::persistence::repositories::user_repo::UserRepo;
use crate::persistence::repositories::session_repo::SessionRepo;
use crate::security::password::PasswordHasherService;

struct LoginTracker {
    failures: std::collections::HashMap<String, Vec<chrono::DateTime<Utc>>>,
    locked_until: std::collections::HashMap<String, chrono::DateTime<Utc>>,
}

impl LoginTracker {
    fn new() -> Self {
        Self {
            failures: std::collections::HashMap::new(),
            locked_until: std::collections::HashMap::new(),
        }
    }

    fn record_failure(&mut self, username: &str) -> u32 {
        let entries = self.failures.entry(username.to_string()).or_default();
        let now = Utc::now();
        // Prune entries older than 30 minutes
        entries.retain(|t| (now - *t).num_minutes() < 30);
        entries.push(now);
        entries.len() as u32
    }

    fn get_delay_ms(&self, attempt_count: u32) -> u64 {
        match attempt_count {
            0 | 1 => 0,
            2 => 100,
            3 => 500,
            4 => 2000,
            _ => 5000,
        }
    }

    fn is_locked(&self, username: &str) -> bool {
        if let Some(until) = self.locked_until.get(username) {
            if Utc::now() < *until {
                return true;
            }
        }
        false
    }

    fn lock_account(&mut self, username: &str) {
        let lock_duration = chrono::Duration::minutes(15);
        self.locked_until.insert(username.to_string(), Utc::now() + lock_duration);
    }

    fn clear_failures(&mut self, username: &str) {
        self.failures.remove(username);
        self.locked_until.remove(username);
    }
}

pub struct AuthService {
    db_pool: Arc<DatabasePool>,
    audit_logger: Arc<AuditLogger>,
    login_tracker: Arc<RwLock<LoginTracker>>,
}

impl AuthService {
    pub fn new(db_pool: Arc<DatabasePool>, audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            db_pool,
            audit_logger,
            login_tracker: Arc::new(RwLock::new(LoginTracker::new())),
        }
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
        workstation: WorkstationInfo,
    ) -> Result<Session, DomainError> {
        // Check lockout
        {
            let tracker = self.login_tracker.read().await;
            if tracker.is_locked(username) {
                warn!(target: "auth", username = %username, "Login attempt on locked account");
                return Err(DomainError::AccountLocked);
            }
        }

        let result = {
            let conn = self.db_pool.connection().await.map_err(|e| {
                DomainError::DatabaseError(e.to_string())
            })?;

            let user = UserRepo::find_by_username(&*conn, username)
                .map_err(|_| DomainError::AuthenticationFailed("Invalid credentials".to_string()))?;

            // Verify account status
            match user.account_status {
                AccountStatus::Locked => {
                    warn!(target: "auth", username = %username, "Login attempt on locked account");
                    let mut tracker = self.login_tracker.write().await;
                    tracker.lock_account(username);
                    return Err(DomainError::AccountLocked);
                }
                AccountStatus::Disabled => {
                    warn!(target: "auth", username = %username, "Login attempt on disabled account");
                    return Err(DomainError::AccountDisabled);
                }
                AccountStatus::Active => {}
            }

            // Verify password
            let valid = PasswordHasherService::verify(password, &user.password_hash)
                .map_err(|_| DomainError::InternalError("Password verification failed".to_string()))?;

            if !valid {
                let mut tracker = self.login_tracker.write().await;
                let attempts = tracker.record_failure(username);

                let delay_ms = tracker.get_delay_ms(attempts);
                if delay_ms > 0 {
                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                }

                if attempts >= 5 {
                    tracker.lock_account(username);
                    let _ = UserRepo::set_account_status(&*conn, user.id, AccountStatus::Locked);
                    warn!(target: "auth", username = %username, attempts = %attempts, "Account locked after max failures");
                }

                // conn dropped here before audit write
                drop(conn);

                let event = AuditEventBuilder::new(AuditEventType::LoginFailure)
                    .with_user_id(user.id)
                    .with_workstation_id(&workstation.workstation_id)
                    .with_metadata(serde_json::json!({
                        "reason": "invalid_password",
                        "attempt": attempts,
                        "hostname": workstation.hostname,
                        "os": workstation.operating_system,
                    }))
                    .build();

                self.audit_logger.write(event).await
                    .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

                return Err(DomainError::AuthenticationFailed("Invalid credentials".to_string()));
            }

            // Clear failure tracking on successful login
            {
                let mut tracker = self.login_tracker.write().await;
                tracker.clear_failures(username);
            }

            // Update last login
            let now = Utc::now();
            let _ = UserRepo::update_last_login(&*conn, user.id, now);

            // Create session
            let session = Session {
                id: Uuid::new_v4(),
                user_id: user.id,
                workstation_id: workstation.workstation_id.clone(),
                hostname: workstation.hostname.clone(),
                operating_system: workstation.operating_system.clone(),
                login_timestamp: now,
                last_activity_timestamp: now,
                revoked_at: None,
                integrity_state: IntegrityState::Intact,
            };

            SessionRepo::create(&*conn, &session)
                .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

            // conn and user moved into session, dropped here before audit write
            drop(conn);

            let event = AuditEventBuilder::new(AuditEventType::LoginSuccess)
                .with_user_id(user.id)
                .with_session_id(session.id)
                .with_workstation_id(&workstation.workstation_id)
                .with_metadata(serde_json::json!({
                    "hostname": workstation.hostname,
                    "os": workstation.operating_system,
                    "role": user.role.as_str(),
                }))
                .build();

            self.audit_logger.write(event).await
                .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

            info!(target: "auth", username = %username, session_id = %session.id, "Login successful");

            session
        };

        Ok(result)
    }

    pub async fn logout(&self, session_id: Uuid) -> Result<(), DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let session = SessionRepo::find_by_id(&*conn, session_id)
            .map_err(|_| DomainError::SessionNotFound(session_id.to_string()))?;

        if session.revoked_at.is_some() {
            return Ok(()); // Already revoked
        }

        let now = Utc::now();
        SessionRepo::revoke(&*conn, session_id, now)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        info!(target: "auth", session_id = %session_id, "Session ended by logout");

        Ok(())
    }

    pub async fn revoke_session(&self, session_id: Uuid, revoked_by: Uuid) -> Result<(), DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let session = SessionRepo::find_by_id(&*conn, session_id)
            .map_err(|_| DomainError::SessionNotFound(session_id.to_string()))?;

        if session.revoked_at.is_some() {
            return Err(DomainError::SessionNotFound("Session already revoked".to_string()));
        }

        let now = Utc::now();
        SessionRepo::revoke(&*conn, session_id, now)
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        // Audit revocation
        let event = AuditEventBuilder::new(AuditEventType::SessionRevoked)
            .with_user_id(session.user_id)
            .with_session_id(session_id)
            .with_metadata(serde_json::json!({
                "revoked_by": revoked_by.to_string(),
                "reason": "manual_revocation",
            }))
            .build();

        self.audit_logger.write(event).await
            .map_err(|e| DomainError::AuditWriteFailed(e.to_string()))?;

        info!(target: "auth", session_id = %session_id, revoked_by = %revoked_by, "Session revoked");

        Ok(())
    }

    pub async fn validate_session(&self, session_id: Uuid) -> Result<Session, DomainError> {
        let conn = self.db_pool.connection().await
            .map_err(|e| DomainError::DatabaseError(e.to_string()))?;

        let session = SessionRepo::find_by_id(&*conn, session_id)
            .map_err(|_| DomainError::SessionNotFound(session_id.to_string()))?;

        // Check if revoked
        if session.revoked_at.is_some() {
            return Err(DomainError::SessionExpired);
        }

        // Check session timeout (default 8 hours)
        let max_inactivity = chrono::Duration::hours(8);
        let elapsed = Utc::now() - session.last_activity_timestamp;
        if elapsed > max_inactivity {
            return Err(DomainError::SessionExpired);
        }

        // Update last activity
        let _ = SessionRepo::update_activity(&*conn, session_id, Utc::now());

        Ok(session)
    }
}
