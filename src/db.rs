use anyhow::Result;
use rusqlite::{Connection, params};
use std::sync::Mutex;

/// Thread-safe wrapper around a single SQLite connection.
/// All blocking calls must be dispatched via `tokio::task::spawn_blocking`.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }
}

// ---------------------------------------------------------------------------
// Repository helpers
// ---------------------------------------------------------------------------

impl Database {
    // --- Users ---

    pub fn create_user(
        &self,
        id: &str,
        username: &str,
        password_hash: &str,
        api_key_hash: &str,
        created_at: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO users (id, username, display_name, password_hash, api_key_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, username, username, password_hash, api_key_hash, created_at],
        )?;
        Ok(())
    }
    
    pub fn find_user_by_id(&self, id: &str) -> Result<Option<UserRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, api_key_hash, created_at, display_name
             FROM users WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(UserRow {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                api_key_hash: row.get(3)?,
                created_at: row.get(4)?,
                display_name: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn find_user_by_username(&self, username: &str) -> Result<Option<UserRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, api_key_hash, created_at, display_name
             FROM users WHERE username = ?1",
        )?;
        let mut rows = stmt.query(params![username])?;
        if let Some(row) = rows.next()? {
            Ok(Some(UserRow {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                api_key_hash: row.get(3)?,
                created_at: row.get(4)?,
                display_name: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn find_user_by_api_key_hash(&self, hash: &str) -> Result<Option<UserRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, api_key_hash, created_at, display_name
             FROM users WHERE api_key_hash = ?1",
        )?;
        let mut rows = stmt.query(params![hash])?;
        if let Some(row) = rows.next()? {
            Ok(Some(UserRow {
                id: row.get(0)?,
                username: row.get(1)?,
                password_hash: row.get(2)?,
                api_key_hash: row.get(3)?,
                created_at: row.get(4)?,
                display_name: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn update_display_name(&self, user_id: &str, display_name: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET display_name = ?1 WHERE id = ?2",
            params![display_name, user_id],
        )?;
        Ok(())
    }

    pub fn update_api_key_hash(&self, user_id: &str, new_hash: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET api_key_hash = ?1 WHERE id = ?2",
            params![new_hash, user_id],
        )?;
        Ok(())
    }

    pub fn update_password(&self, user_id: &str, new_password_hash: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE users SET password_hash = ?1 WHERE id = ?2",
            params![new_password_hash, user_id],
        )?;
        Ok(())
    }

    // --- Sessions ---

    pub fn create_session(
        &self,
        id: &str,
        user_id: &str,
        game_mode: &str,
        started_at: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO sessions (id, user_id, game_mode, started_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![id, user_id, game_mode, started_at],
        )?;
        Ok(())
    }

    pub fn end_session(&self, session_id: &str, ended_at: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
            params![ended_at, session_id],
        )?;
        Ok(())
    }

    // --- High scores ---

    pub fn upsert_high_score(
        &self,
        id: &str,
        user_id: &str,
        session_id: &str,
        max_height: f64,
        achieved_at: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO high_scores (id, user_id, session_id, max_height, achieved_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET max_height = excluded.max_height,
                                           achieved_at = excluded.achieved_at",
            params![id, user_id, session_id, max_height, achieved_at],
        )?;
        Ok(())
    }

    pub fn get_leaderboard(
        &self,
        session_type: Option<&str>,
        stage_type: Option<&str>,
        session_id: Option<&str>,
        user_id: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<LeaderboardRow>> {
        let conn = self.conn.lock().unwrap();

        let mut conditions: Vec<String> = Vec::new();
        if session_id.is_some() {
            conditions.push("h.session_id = :session_id".into());
        } else {
            if session_type.is_some() {
                conditions.push("s.session_type = :session_type".into());
            }
            if stage_type.is_some() {
                conditions.push("s.stage_type = :stage_type".into());
            }
        }
        if user_id.is_some() {
            conditions.push("h.user_id = :user_id".into());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let sql = format!(
            "SELECT u.username, h.max_height, s.game_mode, h.achieved_at
             FROM high_scores h
             JOIN users u ON u.id = h.user_id
             JOIN sessions s ON s.id = h.session_id
             {where_clause}
             ORDER BY h.max_height DESC
             LIMIT :limit OFFSET :offset"
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            rusqlite::named_params! {
                ":session_type": session_type.unwrap_or(""),
                ":stage_type": stage_type.unwrap_or(""),
                ":session_id": session_id.unwrap_or(""),
                ":user_id": user_id.unwrap_or(""),
                ":limit": limit,
                ":offset": offset,
            },
            |row| {
                Ok(LeaderboardRow {
                    username: row.get(0)?,
                    max_height: row.get(1)?,
                    game_mode: row.get(2)?,
                    achieved_at: row.get(3)?,
                })
            },
        )?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

// ---------------------------------------------------------------------------
// Row types (plain data, no domain logic)
// ---------------------------------------------------------------------------

pub struct UserRow {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub api_key_hash: String,
    pub created_at: String,
    pub display_name: Option<String>,
}

pub struct LeaderboardRow {
    pub username: String,
    pub max_height: f64,
    pub game_mode: String,
    pub achieved_at: String,
}
