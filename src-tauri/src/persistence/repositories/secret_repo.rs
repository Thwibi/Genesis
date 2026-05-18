use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::entities::mfa_secret::MfaSecret;

pub struct SecretRepo;

impl SecretRepo {
    pub fn find_active_by_platform(conn: &Connection, platform_id: Uuid) -> Result<MfaSecret, String> {
        conn.query_row(
            "SELECT id, platform_id, encrypted_secret, algorithm, digits, period, created_at, rotated_at, archived_at
             FROM mfa_secrets WHERE platform_id = ?1 AND archived_at IS NULL
             ORDER BY created_at DESC LIMIT 1",
            params![platform_id.to_string()],
            |row| {
                let id_str: String = row.get(0)?;
                let pid_str: String = row.get(1)?;
                let created_str: String = row.get(6)?;
                let rotated_str: Option<String> = row.get(7)?;
                let archived_str: Option<String> = row.get(8)?;

                Ok(MfaSecret {
                    id: Uuid::parse_str(&id_str).unwrap(),
                    platform_id: Uuid::parse_str(&pid_str).unwrap(),
                    encrypted_secret: row.get(2)?,
                    algorithm: row.get(3)?,
                    digits: row.get::<_, i32>(4)? as u32,
                    period: row.get::<_, i64>(5)? as u64,
                    created_at: DateTime::parse_from_rfc3339(&created_str).unwrap().with_timezone(&Utc),
                    rotated_at: rotated_str.map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                    archived_at: archived_str.map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                })
            },
        ).map_err(|e| format!("Secret not found: {}", e))
    }

    pub fn create(conn: &Connection, secret: &MfaSecret) -> Result<(), String> {
        conn.execute(
            "INSERT INTO mfa_secrets (id, platform_id, encrypted_secret, algorithm, digits, period, created_at, rotated_at, archived_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                secret.id.to_string(),
                secret.platform_id.to_string(),
                secret.encrypted_secret,
                secret.algorithm,
                secret.digits as i32,
                secret.period as i64,
                secret.created_at.to_rfc3339(),
                secret.rotated_at.map(|t| t.to_rfc3339()),
                secret.archived_at.map(|t| t.to_rfc3339()),
            ],
        ).map_err(|e| format!("Failed to create secret: {}", e))?;
        Ok(())
    }

    pub fn update_secret(conn: &Connection, platform_id: Uuid, encrypted: &[u8], rotated_at: DateTime<Utc>) -> Result<(), String> {
        conn.execute(
            "UPDATE mfa_secrets SET encrypted_secret = ?1, rotated_at = ?2 WHERE platform_id = ?3 AND archived_at IS NULL",
            params![encrypted, rotated_at.to_rfc3339(), platform_id.to_string()],
        ).map_err(|e| format!("Failed to update secret: {}", e))?;
        Ok(())
    }

    pub fn archive(conn: &Connection, platform_id: Uuid, archived_at: DateTime<Utc>) -> Result<(), String> {
        conn.execute(
            "UPDATE mfa_secrets SET archived_at = ?1 WHERE platform_id = ?2 AND archived_at IS NULL",
            params![archived_at.to_rfc3339(), platform_id.to_string()],
        ).map_err(|e| format!("Failed to archive secret: {}", e))?;
        Ok(())
    }
}
