use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::entities::audit_event::AuditEvent;

pub struct AuditRepo;

impl AuditRepo {
    pub fn insert(conn: &Connection, event: &AuditEvent) -> Result<(), String> {
        conn.execute(
            "INSERT INTO audit_events (id, user_id, session_id, platform_id, event_type, severity, workstation_id, timestamp, integrity_hash, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                event.id.to_string(),
                event.user_id.map(|u| u.to_string()),
                event.session_id.map(|u| u.to_string()),
                event.platform_id.map(|u| u.to_string()),
                event.event_type,
                event.severity,
                event.workstation_id,
                event.timestamp.to_rfc3339(),
                event.integrity_hash,
                serde_json::to_string(&event.metadata).unwrap_or_default(),
            ],
        ).map_err(|e| format!("Failed to insert audit event: {}", e))?;
        Ok(())
    }

    pub fn get_latest_integrity_hash(conn: &Connection) -> Result<Option<String>, String> {
        let result: Result<String, _> = conn.query_row(
            "SELECT integrity_hash FROM audit_events ORDER BY timestamp DESC, rowid DESC LIMIT 1",
            [],
            |row| row.get(0),
        );
        match result {
            Ok(hash) => Ok(Some(hash)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to get latest hash: {}", e)),
        }
    }

    pub fn query(
        conn: &Connection,
        event_type: Option<&str>,
        user_id: Option<Uuid>,
        severity: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuditEvent>, String> {
        let mut sql = String::from(
            "SELECT id, user_id, session_id, platform_id, event_type, severity, workstation_id, timestamp, integrity_hash, metadata
             FROM audit_events WHERE 1=1"
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(et) = event_type {
            sql.push_str(" AND event_type = ?");
            param_values.push(Box::new(et.to_string()));
        }
        if let Some(uid) = user_id {
            sql.push_str(" AND user_id = ?");
            param_values.push(Box::new(uid.to_string()));
        }
        if let Some(sev) = severity {
            sql.push_str(" AND severity = ?");
            param_values.push(Box::new(sev.to_string()));
        }

        sql.push_str(" ORDER BY timestamp DESC LIMIT ? OFFSET ?");
        param_values.push(Box::new(limit));
        param_values.push(Box::new(offset));

        let params_ref: Vec<&dyn rusqlite::types::ToSql> = param_values.iter().map(|p| p.as_ref()).collect();

        let mut stmt = conn.prepare(&sql)
            .map_err(|e| format!("Query preparation failed: {}", e))?;

        let events = stmt.query_map(params_ref.as_slice(), |row| {
            let id_str: String = row.get(0)?;
            let uid_str: Option<String> = row.get(1)?;
            let sid_str: Option<String> = row.get(2)?;
            let pid_str: Option<String> = row.get(3)?;
            let ts_str: String = row.get(7)?;
            let meta_str: String = row.get(9)?;

            Ok(AuditEvent {
                id: Uuid::parse_str(&id_str).unwrap(),
                user_id: uid_str.map(|s| Uuid::parse_str(&s).unwrap()),
                session_id: sid_str.map(|s| Uuid::parse_str(&s).unwrap()),
                platform_id: pid_str.map(|s| Uuid::parse_str(&s).unwrap()),
                event_type: row.get(4)?,
                severity: row.get(5)?,
                workstation_id: row.get(6)?,
                timestamp: DateTime::parse_from_rfc3339(&ts_str).unwrap().with_timezone(&Utc),
                integrity_hash: row.get(8)?,
                metadata: serde_json::from_str(&meta_str).unwrap_or(serde_json::Value::Null),
            })
        }).map_err(|e| format!("Query failed: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

        Ok(events)
    }

    pub fn find_by_id(conn: &Connection, event_id: Uuid) -> Result<AuditEvent, String> {
        conn.query_row(
            "SELECT id, user_id, session_id, platform_id, event_type, severity, workstation_id, timestamp, integrity_hash, metadata
             FROM audit_events WHERE id = ?1",
            params![event_id.to_string()],
            |row| {
                let id_str: String = row.get(0)?;
                let uid_str: Option<String> = row.get(1)?;
                let sid_str: Option<String> = row.get(2)?;
                let pid_str: Option<String> = row.get(3)?;
                let ts_str: String = row.get(7)?;
                let meta_str: String = row.get(9)?;

                Ok(AuditEvent {
                    id: Uuid::parse_str(&id_str).unwrap(),
                    user_id: uid_str.map(|s| Uuid::parse_str(&s).unwrap()),
                    session_id: sid_str.map(|s| Uuid::parse_str(&s).unwrap()),
                    platform_id: pid_str.map(|s| Uuid::parse_str(&s).unwrap()),
                    event_type: row.get(4)?,
                    severity: row.get(5)?,
                    workstation_id: row.get(6)?,
                    timestamp: DateTime::parse_from_rfc3339(&ts_str).unwrap().with_timezone(&Utc),
                    integrity_hash: row.get(8)?,
                    metadata: serde_json::from_str(&meta_str).unwrap_or(serde_json::Value::Null),
                })
            },
        ).map_err(|e| format!("Event not found: {}", e))
    }

    pub fn count_events_by_user_since(
        conn: &Connection,
        user_id: Uuid,
        event_type: &str,
        since: DateTime<Utc>,
    ) -> Result<i64, String> {
        conn.query_row(
            "SELECT COUNT(*) FROM audit_events WHERE user_id = ?1 AND event_type = ?2 AND timestamp >= ?3",
            params![user_id.to_string(), event_type, since.to_rfc3339()],
            |row| row.get(0),
        ).map_err(|e| format!("Count query failed: {}", e))
    }

    pub fn count_platform_access_since(
        conn: &Connection,
        user_id: Uuid,
        platform_id: Uuid,
        since: DateTime<Utc>,
    ) -> Result<i64, String> {
        conn.query_row(
            "SELECT COUNT(*) FROM audit_events WHERE user_id = ?1 AND platform_id = ?2 AND timestamp >= ?3 AND event_type IN ('OTP_GENERATED', 'OTP_COPIED')",
            params![user_id.to_string(), platform_id.to_string(), since.to_rfc3339()],
            |row| row.get(0),
        ).map_err(|e| format!("Count query failed: {}", e))
    }
}
