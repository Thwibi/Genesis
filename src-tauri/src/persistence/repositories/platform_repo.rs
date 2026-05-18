use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::entities::platform::{CriticalityLevel, ProtectedPlatform};

pub struct PlatformRepo;

impl PlatformRepo {
    pub fn find_all(conn: &Connection) -> Result<Vec<ProtectedPlatform>, String> {
        let mut stmt = conn.prepare(
            "SELECT id, name, platform_type, environment, criticality_level, assigned_teams, created_at, updated_at
             FROM protected_platforms ORDER BY name"
        ).map_err(|e| format!("Query preparation failed: {}", e))?;

        let platforms = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let teams_str: String = row.get(5)?;
            let created_str: String = row.get(6)?;
            let updated_str: String = row.get(7)?;

            Ok(ProtectedPlatform {
                id: Uuid::parse_str(&id_str).unwrap(),
                name: row.get(1)?,
                platform_type: row.get(2)?,
                environment: row.get(3)?,
                criticality_level: CriticalityLevel::from_str(&row.get::<_, String>(4)?).unwrap_or(CriticalityLevel::Medium),
                assigned_teams: serde_json::from_str(&teams_str).unwrap_or_default(),
                created_at: DateTime::parse_from_rfc3339(&created_str).unwrap().with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_str).unwrap().with_timezone(&Utc),
            })
        }).map_err(|e| format!("Query failed: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

        Ok(platforms)
    }

    pub fn find_by_id(conn: &Connection, id: Uuid) -> Result<ProtectedPlatform, String> {
        conn.query_row(
            "SELECT id, name, platform_type, environment, criticality_level, assigned_teams, created_at, updated_at
             FROM protected_platforms WHERE id = ?1",
            params![id.to_string()],
            |row| {
                let id_str: String = row.get(0)?;
                let teams_str: String = row.get(5)?;
                let created_str: String = row.get(6)?;
                let updated_str: String = row.get(7)?;

                Ok(ProtectedPlatform {
                    id: Uuid::parse_str(&id_str).unwrap(),
                    name: row.get(1)?,
                    platform_type: row.get(2)?,
                    environment: row.get(3)?,
                    criticality_level: CriticalityLevel::from_str(&row.get::<_, String>(4)?).unwrap_or(CriticalityLevel::Medium),
                    assigned_teams: serde_json::from_str(&teams_str).unwrap_or_default(),
                    created_at: DateTime::parse_from_rfc3339(&created_str).unwrap().with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&updated_str).unwrap().with_timezone(&Utc),
                })
            },
        ).map_err(|e| format!("Platform not found: {}", e))
    }

    pub fn create(conn: &Connection, platform: &ProtectedPlatform) -> Result<(), String> {
        conn.execute(
            "INSERT INTO protected_platforms (id, name, platform_type, environment, criticality_level, assigned_teams, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                platform.id.to_string(),
                platform.name,
                platform.platform_type,
                platform.environment,
                platform.criticality_level.as_str(),
                serde_json::to_string(&platform.assigned_teams).unwrap_or_default(),
                platform.created_at.to_rfc3339(),
                platform.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| format!("Failed to create platform: {}", e))?;
        Ok(())
    }

    pub fn update(conn: &Connection, platform: &ProtectedPlatform) -> Result<(), String> {
        conn.execute(
            "UPDATE protected_platforms SET name = ?1, platform_type = ?2, environment = ?3, criticality_level = ?4, assigned_teams = ?5, updated_at = ?6 WHERE id = ?7",
            params![
                platform.name,
                platform.platform_type,
                platform.environment,
                platform.criticality_level.as_str(),
                serde_json::to_string(&platform.assigned_teams).unwrap_or_default(),
                platform.updated_at.to_rfc3339(),
                platform.id.to_string(),
            ],
        ).map_err(|e| format!("Failed to update platform: {}", e))?;
        Ok(())
    }

    pub fn delete(conn: &Connection, id: Uuid) -> Result<(), String> {
        conn.execute(
            "DELETE FROM protected_platforms WHERE id = ?1",
            params![id.to_string()],
        ).map_err(|e| format!("Failed to delete platform: {}", e))?;
        Ok(())
    }
}
