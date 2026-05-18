use rusqlite::Connection;
use tracing::info;

pub fn run_migrations(conn: &Connection) -> Result<(), String> {
    info!(target: "database", "Running database migrations");

    // V001: Initial schema
    conn.execute_batch(include_str!("V001__initial_schema.sql"))
        .map_err(|e| format!("Migration V001 failed: {}", e))?;

    info!(target: "database", "All migrations completed successfully");
    Ok(())
}
