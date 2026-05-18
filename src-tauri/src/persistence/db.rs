use std::ops::Deref;
use std::path::Path;
use rusqlite::Connection;
use tokio::sync::{Mutex, MutexGuard};
use tracing::info;

use super::migrations::run_migrations;
use crate::security::crypto::derive_key;

const DB_KEY_SALT: &[u8] = b"genesis-db-key-v1";
const DB_KEY_INFO: &[u8] = b"sqlcipher-encryption";

/// A `Connection` wrapper that implements `Send + Sync`.
///
/// `rusqlite::Connection` is `!Send` because SQLite connections are not thread-safe.
/// We wrap it and assert Send/Sync, which is safe because all access is serialized
/// through `tokio::sync::Mutex`.
pub struct SafeConnection {
    conn: Connection,
}

// SAFETY: All access to the inner Connection is serialized through tokio::sync::Mutex,
// ensuring only one task accesses it at any time.
#[allow(unsafe_code)]
unsafe impl Send for SafeConnection {}
#[allow(unsafe_code)]
unsafe impl Sync for SafeConnection {}

impl Deref for SafeConnection {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

pub struct DatabasePool {
    conn: Mutex<SafeConnection>,
}

impl DatabasePool {
    /// Open or create an encrypted SQLCipher database at the given path.
    ///
    /// `master_key` is the 32-byte vault master key. A derived key is used for
    /// SQLCipher so that the master key is never directly used for DB encryption.
    pub fn open(path: &Path, master_key: &[u8]) -> Result<Self, DatabaseError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DatabaseError::OpenFailed(e.to_string()))?;
        }
        info!(target: "database", path = %path.display(), "Opening database");

        let conn = Connection::open(path)
            .map_err(|e| DatabaseError::OpenFailed(e.to_string()))?;

        // Derive SQLCipher key from master key
        let db_key = derive_key(master_key, DB_KEY_SALT, DB_KEY_INFO, 32)
            .map_err(|e| DatabaseError::OpenFailed(format!("Key derivation failed: {}", e)))?;

        // Enable SQLCipher encryption with derived key
        let key_hex = hex::encode(&db_key);
        conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", key_hex))
            .map_err(|e| DatabaseError::OpenFailed(format!("SQLCipher key setup failed: {}", e)))?;

        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| DatabaseError::OpenFailed(e.to_string()))?;

        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| DatabaseError::OpenFailed(e.to_string()))?;

        Ok(Self { conn: Mutex::new(SafeConnection { conn }) })
    }

    pub async fn connection(&self) -> Result<MutexGuard<'_, SafeConnection>, DatabaseError> {
        Ok(self.conn.lock().await)
    }

    /// Synchronous variant for use in non-async contexts (e.g., main).
    pub fn blocking_connection(&self) -> Result<MutexGuard<'_, SafeConnection>, DatabaseError> {
        Ok(self.conn.blocking_lock())
    }

    pub async fn run_migrations(&self) -> Result<(), DatabaseError> {
        let conn = self.connection().await?;
        run_migrations(&conn)
            .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))
    }

    pub fn run_migrations_sync(&self) -> Result<(), DatabaseError> {
        let conn = self.blocking_connection()?;
        run_migrations(&conn)
            .map_err(|e| DatabaseError::MigrationFailed(e.to_string()))
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    OpenFailed(String),
    MigrationFailed(String),
    ConnectionFailed(String),
    QueryFailed(String),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::OpenFailed(msg) => write!(f, "Database open failed: {}", msg),
            DatabaseError::MigrationFailed(msg) => write!(f, "Migration failed: {}", msg),
            DatabaseError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            DatabaseError::QueryFailed(msg) => write!(f, "Query failed: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {}
