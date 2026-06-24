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
    id             TEXT PRIMARY KEY,
    fall_mode      TEXT NOT NULL CHECK(fall_mode IN ('Checkpoint', 'Hardcore')),
    session_type   TEXT NOT NULL CHECK(session_type IN ('Race', 'Timetrial', 'Infinite')),
    stage_type     TEXT NOT NULL CHECK(stage_type IN ('Generated', 'Fixed')),
    seed           INTEGER,
    goal_height    INTEGER,
    fixed_stage_id TEXT,
    started_at     TEXT NOT NULL,
    ended_at       TEXT
);

CREATE INDEX IF NOT EXISTS idx_sessions_fall_mode    ON sessions(fall_mode);
CREATE INDEX IF NOT EXISTS idx_sessions_session_type ON sessions(session_type);
CREATE INDEX IF NOT EXISTS idx_sessions_stage_type   ON sessions(stage_type);

CREATE TABLE IF NOT EXISTS high_scores (
    id                 TEXT PRIMARY KEY,
    user_id            TEXT NOT NULL REFERENCES users(id),
    session_id         TEXT NOT NULL REFERENCES sessions(id),
    max_height         REAL NOT NULL,
    completion_time_ms INTEGER NULL,
    achieved_at        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_high_scores_user   ON high_scores(user_id);
CREATE INDEX IF NOT EXISTS idx_high_scores_height ON high_scores(max_height DESC);
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
