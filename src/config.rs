use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// Host:port the HTTP server binds to.
    pub bind: String,
    /// Path to the built frontend dist folder served as static files.
    pub frontend_dist_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file.
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    /// Secret used to sign and verify JWTs. Must be at least 32 characters.
    pub jwt_secret: String,
    /// JWT expiry in seconds (default 3600).
    pub jwt_expiry_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitingConfig {
    /// Max requests per second per IP (global).
    pub ip_requests_per_second: u32,
    /// Burst allowance for IP-level limiting.
    pub ip_burst_size: u32,
    /// Max requests per second per bot API key.
    pub bot_requests_per_second: u32,
    /// Burst allowance for bot-level limiting.
    pub bot_burst_size: u32,
    /// Max requests per second per user JWT.
    pub user_requests_per_second: u32,
    /// Burst allowance for user-level limiting.
    pub user_burst_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GameConfig {
    /// Default game mode on startup ("Checkpoint" or "Hardcore").
    pub default_game_mode: String,
    /// Default session type on startup ("Race", "Timetrial", "Infinite").
    pub default_session_type: String,
    /// Default stage type on startup ("Generated", "Fixed").
    pub default_stage_type: String,
    /// Default fixed stage ID if stage_type is Fixed.
    pub default_stage_id: Option<String>,
    /// Seconds of inactivity before a player session is cleaned up.
    pub player_timeout_secs: u64,
    /// Maximum number of concurrent players.
    pub max_players: u32,
    /// Physics tick rate in Hz.
    pub tick_rate_hz: f64,
    /// Path to a folder containing stage definition TOML files.
    pub maps_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub rate_limiting: RateLimitingConfig,
    pub game: GameConfig,
    #[serde(skip)]
    pub fixed_stages: Vec<crate::models::FixedStage>,
}

impl AppConfig {
    pub fn load(config_path: &str) -> Result<Self> {
        let cfg = config::Config::builder()
            // [server]
            .set_default("server.bind", "0.0.0.0:8080")?
            .set_default("server.frontend_dist_path", "frontend/dist")?
            // [database]
            .set_default("database.path", ".data/botclimber.db")?
            // [auth]
            .set_default("auth.jwt_secret", "")?
            .set_default("auth.jwt_expiry_secs", 3600_i64)?
            // [rate_limiting]
            .set_default("rate_limiting.ip_requests_per_second", 100_i64)?
            .set_default("rate_limiting.ip_burst_size", 200_i64)?
            .set_default("rate_limiting.bot_requests_per_second", 10_i64)?
            .set_default("rate_limiting.bot_burst_size", 20_i64)?
            .set_default("rate_limiting.user_requests_per_second", 30_i64)?
            .set_default("rate_limiting.user_burst_size", 60_i64)?
            // [game]
            .set_default("game.default_game_mode", "Hardcore")?
            .set_default("game.default_session_type", "Infinite")?
            .set_default("game.default_stage_type", "Generated")?
            .set_default("game.player_timeout_secs", 60_i64)?
            .set_default("game.max_players", 50_i64)?
            .set_default("game.tick_rate_hz", 60.0_f64)?
            // Config file passed as CLI argument
            .add_source(
                config::File::with_name(config_path)
                    .format(config::FileFormat::Toml)
                    .required(false),
            )
            // Environment variable overrides (e.g. BOTCLIMBER__AUTH__JWT_SECRET)
            .add_source(
                config::Environment::with_prefix("BOTCLIMBER")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        let mut app_cfg: AppConfig = cfg.try_deserialize()?;
        app_cfg.load_fixed_stages()?;
        app_cfg.validate()?;
        Ok(app_cfg)
    }

    fn load_fixed_stages(&mut self) -> Result<()> {
        if let Some(ref maps_path) = self.game.maps_path {
            let path = std::path::Path::new(maps_path);
            if !path.exists() {
                // If the path doesn't exist, we just leave fixed_stages empty.
                // Alternatively, we could log a warning or bail.
                return Ok(());
            }

            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    let content = std::fs::read_to_string(&path)?;
                    let mut stage: crate::models::FixedStage = toml::from_str(&content)?;
                    // Set the stage ID based on the filename (sans extension) if not explicitly set
                    if stage.id.is_empty() {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            stage.id = stem.to_string();
                        }
                    }
                    self.fixed_stages.push(stage);
                }
            }
        }
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        if self.auth.jwt_secret.is_empty() {
            anyhow::bail!(
                "auth.jwt_secret (or BOTCLIMBER__AUTH__JWT_SECRET) must be set before starting the server"
            );
        }
        if self.auth.jwt_secret.len() < 32 {
            anyhow::bail!("auth.jwt_secret must be at least 32 characters long");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_load_fixed_stages_from_folder() {
        let temp_dir = std::env::temp_dir().join(format!("botclimber_maps_{}", Uuid::new_v4()));
        std::fs::create_dir(&temp_dir).unwrap();

        let map1_content = r#"
name = "Map 1"
goal_height = 50.0
platforms = [
    { bottom_left = [0.0, 0.0], top_right = [10.0, 2.0] }
]
"#;
        let map2_content = r#"
id = "explicit_id"
name = "Map 2"
goal_height = 100.0
platforms = [
    { bottom_left = [5.0, 5.0], top_right = [15.0, 7.0] }
]
"#;

        std::fs::write(temp_dir.join("map1.toml"), map1_content).unwrap();
        std::fs::write(temp_dir.join("map2.toml"), map2_content).unwrap();
        // Non-toml file should be ignored
        std::fs::write(temp_dir.join("ignore.txt"), "hello").unwrap();

        let mut app_cfg = AppConfig {
            server: ServerConfig { bind: "".into(), frontend_dist_path: "".into() },
            database: DatabaseConfig { path: "".into() },
            auth: AuthConfig { jwt_secret: "".into(), jwt_expiry_secs: 0 },
            rate_limiting: RateLimitingConfig {
                ip_requests_per_second: 0, ip_burst_size: 0,
                bot_requests_per_second: 0, bot_burst_size: 0,
                user_requests_per_second: 0, user_burst_size: 0,
            },
            game: GameConfig {
                default_game_mode: "".into(),
                default_session_type: "Infinite".into(),
                default_stage_type: "Generated".into(),
                default_stage_id: None,
                player_timeout_secs: 0,
                max_players: 0,
                tick_rate_hz: 0.0,
                maps_path: Some(temp_dir.to_str().unwrap().to_string()),
            },
            fixed_stages: Vec::new(),
        };

        app_cfg.load_fixed_stages().unwrap();

        assert_eq!(app_cfg.fixed_stages.len(), 2);

        let m1 = app_cfg.fixed_stages.iter().find(|s| s.name == "Map 1").unwrap();
        assert_eq!(m1.id, "map1");
        assert_eq!(m1.goal_height, 50.0);

        let m2 = app_cfg.fixed_stages.iter().find(|s| s.name == "Map 2").unwrap();
        assert_eq!(m2.id, "explicit_id"); // explicitly set in file
        assert_eq!(m2.goal_height, 100.0);

        std::fs::remove_dir_all(temp_dir).unwrap();
    }
}
