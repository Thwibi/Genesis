use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::domain::entities::user::{AccountStatus, User, UserRole};

pub struct UserRepo;

impl UserRepo {
    pub fn find_by_username(conn: &Connection, username: &str) -> Result<User, String> {
        conn.query_row(
            "SELECT id, username, email, password_hash, role, assigned_shift_id, account_status, created_at, last_login_at
             FROM users WHERE username = ?1",
            params![username],
            |row| {
                let id_str: String = row.get(0)?;
                let shift_id_str: Option<String> = row.get(5)?;
                let created_str: String = row.get(7)?;
                let last_login_str: Option<String> = row.get(8)?;

                Ok(User {
                    id: Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: UserRole::from_str(&row.get::<_, String>(4)?)
                        .ok_or_else(|| rusqlite::Error::InvalidColumnType(4, String::new(), rusqlite::types::Type::Text))?,
                    assigned_shift_id: shift_id_str.map(|s| Uuid::parse_str(&s).unwrap()),
                    account_status: AccountStatus::from_str(&row.get::<_, String>(6)?)
                        .ok_or_else(|| rusqlite::Error::InvalidColumnType(6, String::new(), rusqlite::types::Type::Text))?,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                        .with_timezone(&Utc),
                    last_login_at: last_login_str.map(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now())
                    }),
                })
            },
        ).map_err(|e| format!("User not found: {}", e))
    }

    pub fn find_by_id(conn: &Connection, id: Uuid) -> Result<User, String> {
        conn.query_row(
            "SELECT id, username, email, password_hash, role, assigned_shift_id, account_status, created_at, last_login_at
             FROM users WHERE id = ?1",
            params![id.to_string()],
            |row| {
                let id_str: String = row.get(0)?;
                let shift_id_str: Option<String> = row.get(5)?;
                let created_str: String = row.get(7)?;
                let last_login_str: Option<String> = row.get(8)?;

                Ok(User {
                    id: Uuid::parse_str(&id_str).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    password_hash: row.get(3)?,
                    role: UserRole::from_str(&row.get::<_, String>(4)?)
                        .ok_or_else(|| rusqlite::Error::InvalidColumnType(4, String::new(), rusqlite::types::Type::Text))?,
                    assigned_shift_id: shift_id_str.map(|s| Uuid::parse_str(&s).unwrap()),
                    account_status: AccountStatus::from_str(&row.get::<_, String>(6)?)
                        .ok_or_else(|| rusqlite::Error::InvalidColumnType(6, String::new(), rusqlite::types::Type::Text))?,
                    created_at: DateTime::parse_from_rfc3339(&created_str)
                        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                        .with_timezone(&Utc),
                    last_login_at: last_login_str.map(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now())
                    }),
                })
            },
        ).map_err(|e| format!("User not found: {}", e))
    }

    pub fn set_account_status(conn: &Connection, id: Uuid, status: AccountStatus) -> Result<(), String> {
        conn.execute(
            "UPDATE users SET account_status = ?1 WHERE id = ?2",
            params![status.as_str(), id.to_string()],
        ).map_err(|e| format!("Failed to update account status: {}", e))?;
        Ok(())
    }

    pub fn update_last_login(conn: &Connection, id: Uuid, timestamp: DateTime<Utc>) -> Result<(), String> {
        conn.execute(
            "UPDATE users SET last_login_at = ?1 WHERE id = ?2",
            params![timestamp.to_rfc3339(), id.to_string()],
        ).map_err(|e| format!("Failed to update last login: {}", e))?;
        Ok(())
    }

    pub fn count(conn: &Connection) -> Result<i64, String> {
        conn.query_row(
            "SELECT COUNT(*) FROM users",
            [],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to count users: {}", e))
    }

    pub fn create(conn: &Connection, user: &User) -> Result<(), String> {
        conn.execute(
            "INSERT INTO users (id, username, email, password_hash, role, assigned_shift_id, account_status, created_at, last_login_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                user.id.to_string(),
                user.username,
                user.email,
                user.password_hash,
                user.role.as_str(),
                user.assigned_shift_id.map(|u| u.to_string()),
                user.account_status.as_str(),
                user.created_at.to_rfc3339(),
                user.last_login_at.map(|t| t.to_rfc3339()),
            ],
        ).map_err(|e| format!("Failed to create user: {}", e))?;
        Ok(())
    }
}
