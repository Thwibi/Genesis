#![deny(unsafe_code)]

pub mod commands;
pub mod domain;
pub mod security;
pub mod audit;
pub mod persistence;
pub mod session;
pub mod config;

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::domain::services::auth_service::AuthService;
use crate::domain::services::otp_service::OtpService;
use crate::domain::services::platform_service::PlatformService;
use crate::domain::services::secret_service::SecretService;
use crate::domain::services::shift_service::ShiftService;
use crate::domain::services::monitoring_service::MonitoringService;
use crate::persistence::db::DatabasePool;
use crate::audit::logger::AuditLogger;

pub struct AppState {
    pub auth_service: Arc<RwLock<AuthService>>,
    pub otp_service: Arc<RwLock<OtpService>>,
    pub platform_service: Arc<RwLock<PlatformService>>,
    pub secret_service: Arc<RwLock<SecretService>>,
    pub shift_service: Arc<RwLock<ShiftService>>,
    pub monitoring_service: Arc<RwLock<MonitoringService>>,
    pub audit_logger: Arc<AuditLogger>,
    pub db_pool: Arc<DatabasePool>,
}

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .json()
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "genesis_soc_auth=info".into()),
        )
        .init();

    tracing::info!(target: "startup", "Genesis SOC Auth tracing initialized");
}

pub fn build_app_state(db_pool: DatabasePool, master_key: &[u8]) -> AppState {
    let pool = Arc::new(db_pool);
    let audit_logger = Arc::new(AuditLogger::new(pool.clone()));

    AppState {
        auth_service: Arc::new(RwLock::new(AuthService::new(pool.clone(), audit_logger.clone()))),
        otp_service: Arc::new(RwLock::new(OtpService::new(pool.clone(), audit_logger.clone(), master_key))),
        platform_service: Arc::new(RwLock::new(PlatformService::new(pool.clone(), audit_logger.clone()))),
        secret_service: Arc::new(RwLock::new(SecretService::new(pool.clone(), audit_logger.clone(), master_key))),
        shift_service: Arc::new(RwLock::new(ShiftService::new(pool.clone(), audit_logger.clone()))),
        monitoring_service: Arc::new(RwLock::new(MonitoringService::new(pool.clone(), audit_logger.clone()))),
        audit_logger,
        db_pool: pool,
    }
}
