use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::entities::shift::{Shift, ShiftAssignment};

pub struct ShiftRepo;

impl ShiftRepo {
    pub fn create(conn: &Connection, shift: &Shift) -> Result<(), String> {
        conn.execute(
            "INSERT INTO shifts (id, shift_name, shift_start, shift_end, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                shift.id.to_string(),
                shift.shift_name,
                shift.shift_start.to_rfc3339(),
                shift.shift_end.to_rfc3339(),
                shift.created_at.to_rfc3339(),
            ],
        ).map_err(|e| format!("Failed to create shift: {}", e))?;
        Ok(())
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Shift>, String> {
        let mut stmt = conn.prepare(
            "SELECT id, shift_name, shift_start, shift_end, created_at FROM shifts ORDER BY shift_start"
        ).map_err(|e| format!("Query preparation failed: {}", e))?;

        let shifts = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let start_str: String = row.get(2)?;
            let end_str: String = row.get(3)?;
            let created_str: String = row.get(4)?;

            Ok(Shift {
                id: Uuid::parse_str(&id_str).unwrap(),
                shift_name: row.get(1)?,
                shift_start: DateTime::parse_from_rfc3339(&start_str).unwrap().with_timezone(&Utc),
                shift_end: DateTime::parse_from_rfc3339(&end_str).unwrap().with_timezone(&Utc),
                created_at: DateTime::parse_from_rfc3339(&created_str).unwrap().with_timezone(&Utc),
            })
        }).map_err(|e| format!("Query failed: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

        Ok(shifts)
    }

    pub fn assign_analyst(conn: &Connection, assignment: &ShiftAssignment) -> Result<(), String> {
        conn.execute(
            "INSERT INTO shift_assignments (id, shift_id, user_id, assigned_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                assignment.id.to_string(),
                assignment.shift_id.to_string(),
                assignment.user_id.to_string(),
                assignment.assigned_at.to_rfc3339(),
            ],
        ).map_err(|e| format!("Failed to assign analyst: {}", e))?;
        Ok(())
    }

    pub fn find_assignments_by_shift(conn: &Connection, shift_id: Uuid) -> Result<Vec<ShiftAssignment>, String> {
        let mut stmt = conn.prepare(
            "SELECT id, shift_id, user_id, assigned_at FROM shift_assignments WHERE shift_id = ?1"
        ).map_err(|e| format!("Query preparation failed: {}", e))?;

        let assignments = stmt.query_map(params![shift_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let sid_str: String = row.get(1)?;
            let uid_str: String = row.get(2)?;
            let at_str: String = row.get(3)?;

            Ok(ShiftAssignment {
                id: Uuid::parse_str(&id_str).unwrap(),
                shift_id: Uuid::parse_str(&sid_str).unwrap(),
                user_id: Uuid::parse_str(&uid_str).unwrap(),
                assigned_at: DateTime::parse_from_rfc3339(&at_str).unwrap().with_timezone(&Utc),
            })
        }).map_err(|e| format!("Query failed: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

        Ok(assignments)
    }
}
