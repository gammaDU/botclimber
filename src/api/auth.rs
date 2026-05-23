use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, HeaderMap},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AppError;
use crate::state::AppState;

// ---------------------------------------------------------------------------
// JWT claims
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: user UUID
    pub sub: String,
    /// Username
    pub username: String,
    /// Admin flag
    pub admin: bool,
    /// Expiry (Unix timestamp)
    pub exp: u64,
}

pub fn encode_jwt(
    user_id: &Uuid,
    username: &str,
    admin: bool,
    secret: &str,
    expiry_secs: u64,
) -> Result<String, AppError> {
    let exp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + expiry_secs;

    let claims = Claims {
        sub: user_id.to_string(),
        username: username.to_owned(),
        admin,
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT encode error: {e}")))
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, AppError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized)
}

// ---------------------------------------------------------------------------
// API key generation and hashing
// ---------------------------------------------------------------------------

/// Generate a new random API key (base64url, 32 bytes of entropy).
pub fn generate_api_key() -> String {
    Uuid::new_v4().to_string()
}

/// Hash an API key for storage (SHA-256 via argon2 is overkill for a key;
/// we use a simple SHA-256 hex digest so lookups stay fast).
pub fn hash_api_key(key: &str) -> String {
    use sha2::Digest;
    let hash = sha2::Sha256::digest(key.as_bytes());
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

// ---------------------------------------------------------------------------
// Axum extractors
// ---------------------------------------------------------------------------

/// Extractor: authenticated user from Bearer JWT (Frontend / Admin routes).
pub struct AuthUser(pub Claims);

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<AppState>: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app: Arc<AppState> = FromRef::from_ref(state);
        let token = bearer_token(&parts.headers)?;
        let claims = decode_jwt(token, &app.config.auth.jwt_secret)?;
        Ok(AuthUser(claims))
    }
}

/// Extractor: admin-only Bearer JWT.
pub struct AdminUser(pub Claims);

impl<S> FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
    Arc<AppState>: axum::extract::FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AuthUser(claims) = AuthUser::from_request_parts(parts, state).await?;
        if !claims.admin {
            return Err(AppError::Forbidden);
        }
        Ok(AdminUser(claims))
    }
}

/// Extractor: bot API key from `X-API-Key` header.
pub struct BotApiKey(pub String);

impl<S> FromRequestParts<S> for BotApiKey
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let key = parts
            .headers
            .get("X-API-Key")
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;
        Ok(BotApiKey(key.to_owned()))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn bearer_token(headers: &HeaderMap) -> Result<&str, AppError> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    value.strip_prefix("Bearer ").ok_or(AppError::Unauthorized)
}
