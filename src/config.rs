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
    /// Seconds of inactivity before a player session is cleaned up.
    pub player_timeout_secs: u64,
    /// Maximum number of concurrent players.
    pub max_players: u32,
    /// Physics tick rate in Hz.
    pub tick_rate_hz: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub rate_limiting: RateLimitingConfig,
    pub game: GameConfig,
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
            .set_default("game.default_game_mode", "Checkpoint")?
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

        let app_cfg: AppConfig = cfg.try_deserialize()?;
        app_cfg.validate()?;
        Ok(app_cfg)
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
