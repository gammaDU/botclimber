use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(
            "
CREATE TABLE IF NOT EXISTS users (
    id            TEXT PRIMARY KEY,
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    api_key_hash  TEXT NOT NULL UNIQUE,
    display_name  TEXT,
    is_admin      INTEGER NOT NULL DEFAULT 0,
    created_at    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    user_id     TEXT NOT NULL REFERENCES users(id),
    game_mode   TEXT NOT NULL CHECK(game_mode IN ('Checkpoint', 'Hardcore')),
    started_at  TEXT NOT NULL,
    ended_at    TEXT
);

CREATE TABLE IF NOT EXISTS high_scores (
    id           TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL REFERENCES users(id),
    session_id   TEXT NOT NULL REFERENCES sessions(id),
    max_height   REAL NOT NULL,
    achieved_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_high_scores_user   ON high_scores(user_id);
CREATE INDEX IF NOT EXISTS idx_high_scores_height ON high_scores(max_height DESC);
CREATE INDEX IF NOT EXISTS idx_sessions_user      ON sessions(user_id);
        ",
        ),
        // Migration 2: rework sessions and high_scores to support the new game mode system
        M::up(
            "
-- Rename old sessions table
ALTER TABLE sessions RENAME TO sessions_old;

-- Create new sessions table with full game mode config columns
CREATE TABLE IF NOT EXISTS sessions (
    id             TEXT PRIMARY KEY,
    fall_mode      TEXT NOT NULL CHECK(fall_mode IN ('Checkpoint', 'Hardcore')),
    session_type   TEXT NOT NULL CHECK(session_type IN ('Race', 'Timetrial', 'Infinite')),
    stage_type     TEXT NOT NULL CHECK(stage_type IN ('Generated', 'Fixed')),
    seed           INTEGER,
    goal_height    REAL,
    fixed_stage_id TEXT,
    started_at     TEXT NOT NULL,
    ended_at       TEXT
);

-- Migrate existing rows: map old game_mode to new columns with sensible defaults
INSERT INTO sessions (id, fall_mode, session_type, stage_type, seed, goal_height, fixed_stage_id, started_at, ended_at)
SELECT
    id,
    game_mode,
    'Infinite',
    'Generated',
    NULL,
    NULL,
    NULL,
    started_at,
    ended_at
FROM sessions_old;

DROP TABLE sessions_old;

CREATE INDEX IF NOT EXISTS idx_sessions_fall_mode    ON sessions(fall_mode);
CREATE INDEX IF NOT EXISTS idx_sessions_session_type ON sessions(session_type);
CREATE INDEX IF NOT EXISTS idx_sessions_stage_type   ON sessions(stage_type);

-- Add completion_time_ms to high_scores for Timetrial sessions
ALTER TABLE high_scores ADD COLUMN completion_time_ms INTEGER;
        ",
        ),
        // Add future migrations here as additional M::up(...) entries.
    ])
}

fn main() {
    let db_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".data/botclimber.db".to_string());

    let mut conn = Connection::open(&db_path)
        .unwrap_or_else(|e| panic!("Failed to open database at '{}': {}", db_path, e));

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .expect("Failed to set PRAGMAs");

    migrations()
        .to_latest(&mut conn)
        .unwrap_or_else(|e| panic!("Migration failed: {}", e));

    println!("Migrations applied successfully to '{}'.", db_path);
}
