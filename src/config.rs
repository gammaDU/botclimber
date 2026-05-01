use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Path to the SQLite database file.
    /// Override with BOTCLIMBER_DATABASE_PATH.
    pub database_path: String,

    /// Host:port the HTTP server binds to.
    /// Override with BOTCLIMBER_SERVER_BIND.
    pub server_bind: String,

    /// Secret used to sign and verify JWTs.
    /// Must be set via BOTCLIMBER_AUTH_JWT_SECRET; never hardcode.
    pub auth_jwt_secret: String,

    /// JWT expiry in seconds (default 3600).
    pub auth_jwt_expiry_secs: u64,

    /// Default game mode on startup ("Checkpoint" or "Hardcore").
    pub default_game_mode: String,

    /// Seconds of inactivity before a player session is cleaned up.
    pub player_timeout_secs: u64,

    /// Global rate limit: max requests per second per IP.
    pub rate_limit_global_rps: u32,

    /// Per-API-key rate limit: max requests per second.
    pub rate_limit_bot_rps: u32,

    /// Burst allowance on top of the steady-state rate.
    pub rate_limit_burst: u32,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let cfg = config::Config::builder()
            // Base defaults
            .set_default("database_path", "botclimber.db")?
            .set_default("server_bind", "0.0.0.0:8080")?
            .set_default("auth_jwt_secret", "")?
            .set_default("auth_jwt_expiry_secs", 3600_i64)?
            .set_default("default_game_mode", "Checkpoint")?
            .set_default("player_timeout_secs", 60_i64)?
            .set_default("rate_limit_global_rps", 100_i64)?
            .set_default("rate_limit_bot_rps", 10_i64)?
            .set_default("rate_limit_burst", 20_i64)?
            // Optional config.toml file
            .add_source(
                config::File::with_name("config")
                    .format(config::FileFormat::Toml)
                    .required(false),
            )
            // Environment variable overrides (e.g. BOTCLIMBER_DATABASE_PATH)
            .add_source(
                config::Environment::with_prefix("BOTCLIMBER")
                    .separator("_")
                    .try_parsing(true),
            )
            .build()?;

        let app_cfg: AppConfig = cfg.try_deserialize()?;
        app_cfg.validate()?;
        Ok(app_cfg)
    }

    fn validate(&self) -> Result<()> {
        if self.auth_jwt_secret.is_empty() {
            anyhow::bail!(
                "BOTCLIMBER_AUTH_JWT_SECRET must be set to a non-empty value before starting the server"
            );
        }
        Ok(())
    }
}
