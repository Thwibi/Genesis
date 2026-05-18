use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::entities::session::{IntegrityState, Session};

pub struct SessionRepo;

impl SessionRepo {
    pub fn create(conn: &Connection, session: &Session) -> Result<(), String> {
        conn.execute(
            "INSERT INTO sessions (id, user_id, workstation_id, hostname, operating_system, login_timestamp, last_activity_timestamp, revoked_at, integrity_state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                session.id.to_string(),
                session.user_id.to_string(),
                session.workstation_id,
                session.hostname,
                session.operating_system,
                session.login_timestamp.to_rfc3339(),
                session.last_activity_timestamp.to_rfc3339(),
                session.revoked_at.map(|t| t.to_rfc3339()),
                session.integrity_state.as_str(),
            ],
        ).map_err(|e| format!("Failed to create session: {}", e))?;
        Ok(())
    }

    pub fn find_by_id(conn: &Connection, id: Uuid) -> Result<Session, String> {
        conn.query_row(
            "SELECT id, user_id, workstation_id, hostname, operating_system, login_timestamp, last_activity_timestamp, revoked_at, integrity_state
             FROM sessions WHERE id = ?1",
            params![id.to_string()],
            |row| {
                let id_str: String = row.get(0)?;
                let user_id_str: String = row.get(1)?;
                let login_str: String = row.get(5)?;
                let activity_str: String = row.get(6)?;
                let revoked_str: Option<String> = row.get(7)?;

                Ok(Session {
                    id: Uuid::parse_str(&id_str).unwrap(),
                    user_id: Uuid::parse_str(&user_id_str).unwrap(),
                    workstation_id: row.get(2)?,
                    hostname: row.get(3)?,
                    operating_system: row.get(4)?,
                    login_timestamp: DateTime::parse_from_rfc3339(&login_str).unwrap().with_timezone(&Utc),
                    last_activity_timestamp: DateTime::parse_from_rfc3339(&activity_str).unwrap().with_timezone(&Utc),
                    revoked_at: revoked_str.map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                    integrity_state: IntegrityState::from_str(&row.get::<_, String>(8)?).unwrap_or(IntegrityState::Intact),
                })
            },
        ).map_err(|e| format!("Session not found: {}", e))
    }

    pub fn revoke(conn: &Connection, id: Uuid, revoked_at: DateTime<Utc>) -> Result<(), String> {
        conn.execute(
            "UPDATE sessions SET revoked_at = ?1 WHERE id = ?2",
            params![revoked_at.to_rfc3339(), id.to_string()],
        ).map_err(|e| format!("Failed to revoke session: {}", e))?;
        Ok(())
    }

    pub fn update_activity(conn: &Connection, id: Uuid, timestamp: DateTime<Utc>) -> Result<(), String> {
        conn.execute(
            "UPDATE sessions SET last_activity_timestamp = ?1 WHERE id = ?2",
            params![timestamp.to_rfc3339(), id.to_string()],
        ).map_err(|e| format!("Failed to update activity: {}", e))?;
        Ok(())
    }

    pub fn find_active_by_user(conn: &Connection, user_id: Uuid) -> Result<Vec<Session>, String> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, workstation_id, hostname, operating_system, login_timestamp, last_activity_timestamp, revoked_at, integrity_state
             FROM sessions WHERE user_id = ?1 AND revoked_at IS NULL"
        ).map_err(|e| format!("Query preparation failed: {}", e))?;

        let sessions = stmt.query_map(params![user_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let uid_str: String = row.get(1)?;
            let login_str: String = row.get(5)?;
            let activity_str: String = row.get(6)?;
            let revoked_str: Option<String> = row.get(7)?;

            Ok(Session {
                id: Uuid::parse_str(&id_str).unwrap(),
                user_id: Uuid::parse_str(&uid_str).unwrap(),
                workstation_id: row.get(2)?,
                hostname: row.get(3)?,
                operating_system: row.get(4)?,
                login_timestamp: DateTime::parse_from_rfc3339(&login_str).unwrap().with_timezone(&Utc),
                last_activity_timestamp: DateTime::parse_from_rfc3339(&activity_str).unwrap().with_timezone(&Utc),
                revoked_at: revoked_str.map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                integrity_state: IntegrityState::from_str(&row.get::<_, String>(8)?).unwrap_or(IntegrityState::Intact),
            })
        }).map_err(|e| format!("Query failed: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

        Ok(sessions)
    }
}
