// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use genesis_soc_auth::{
    build_app_state,
    commands::{
        auth::*,
        platforms::*,
        otp::*,
        secrets::*,
        audit::*,
        shifts::*,
        monitoring::*,
        setup::*,
    },
    config::settings::Settings,
    init_tracing,
    persistence::db::DatabasePool,
    security::vault::Vault,
};

fn main() {
    init_tracing();

    tracing::info!(target: "startup", "Genesis SOC Auth starting");

    let settings = Settings::load().expect("Failed to load configuration");

    Vault::initialize(&settings.master_key_path)
        .expect("Failed to initialize vault");

    // Vault integrity verification — validates master key file checksum
    Vault::verify_integrity_at(&settings.master_key_path)
        .expect("Vault integrity check failed");

    let master_key = Vault::load_master_key(&settings.master_key_path)
        .expect("Failed to load master key");

    let db_pool = DatabasePool::open(&settings.database_path, &master_key)
        .expect("Failed to open database");

    db_pool.run_migrations_sync()
        .expect("Failed to run database migrations");

    let app_state = build_app_state(db_pool, &master_key);

    tracing::info!(target: "startup", "Genesis SOC Auth initialized successfully");

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            cmd_login,
            cmd_logout,
            cmd_revoke_session,
            cmd_validate_session,
            cmd_list_platforms,
            cmd_create_platform,
            cmd_update_platform,
            cmd_delete_platform,
            cmd_generate_otp,
            cmd_copy_otp,
            cmd_store_secret,
            cmd_rotate_secret,
            cmd_archive_secret,
            cmd_query_audit_logs,
            cmd_get_audit_event,
            cmd_create_shift,
            cmd_assign_analyst,
            cmd_list_shifts,
            cmd_get_active_shifts,
            cmd_get_security_alerts,
            cmd_acknowledge_alert,
            cmd_check_users,
            cmd_create_first_user,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running Genesis SOC Auth");
}
