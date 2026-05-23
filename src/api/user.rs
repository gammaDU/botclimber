use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::auth::{AuthUser, encode_jwt, generate_api_key, hash_api_key};
use crate::error::{ApiResult, AppError};
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/profile", get(profile).patch(update_profile))
        .route("/apikey", post(regenerate_apikey))
}

// ---------------------------------------------------------------------------
// POST /api/v1/user/register
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    user_id: Uuid,
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RegisterRequest>,
) -> ApiResult<(StatusCode, Json<RegisterResponse>)> {
    let password_hash = hash_password(&body.password)?;

    let api_key = generate_api_key();
    let api_key_hash = hash_api_key(&api_key);
    let user_id = Uuid::new_v4();
    let now = chrono::Utc::now().to_rfc3339();

    let id_str = user_id.to_string();
    let username = body.username.clone();

    state
        .db
        .create_user(&id_str, &username, &password_hash, &api_key_hash, &now)
        .map_err(|_| AppError::Conflict("username already taken".into()))?;

    Ok((StatusCode::CREATED, Json(RegisterResponse { user_id })))
}

fn hash_password(password: &str) -> Result<String, AppError> {
    use argon2::{
        Argon2, PasswordHasher,
        password_hash::{SaltString, rand_core::OsRng},
    };

    {
        let salt = SaltString::generate(&mut OsRng);
        Ok(Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("argon2: {e}")))?
            .to_string())
    }
}

// ---------------------------------------------------------------------------
// POST /api/v1/user/login
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> ApiResult<Json<LoginResponse>> {
    use argon2::{Argon2, PasswordVerifier, password_hash::PasswordHash};

    let row = state
        .db
        .find_user_by_username(&body.username)?
        .ok_or(AppError::Unauthorized)?;

    let parsed_hash = PasswordHash::new(&row.password_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("hash parse: {e}")))?;

    Argon2::default()
        .verify_password(body.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized)?;

    let user_id: Uuid = row
        .id
        .parse()
        .map_err(|_| AppError::Internal(anyhow::anyhow!("bad uuid")))?;
    let token = encode_jwt(
        &user_id,
        &row.username,
        false,
        &state.config.auth.jwt_secret,
        state.config.auth.jwt_expiry_secs,
    )?;

    Ok(Json(LoginResponse { token }))
}

// ---------------------------------------------------------------------------
// GET /api/v1/user/profile
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ProfileResponse {
    id: String,
    username: String,
    display_name: String,
}

async fn profile(
    AuthUser(claims): AuthUser,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ProfileResponse>> {
    let user = state.db.find_user_by_username(claims.username.as_str())?;
    Ok(Json(ProfileResponse {
        id: claims.sub,
        username: claims.username.clone(),
        display_name: user
            .as_ref()
            .and_then(|u| u.display_name.clone())
            .unwrap_or(claims.username),
    }))
}

// ---------------------------------------------------------------------------
// PATCH /api/v1/user/profile
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct UpdateProfileRequest {
    display_name: Option<String>,
    password: Option<String>,
}

async fn update_profile(
    AuthUser(claims): AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateProfileRequest>,
) -> ApiResult<StatusCode> {
    if let Some(new_display_name) = body.display_name {
        state
            .db
            .update_display_name(&claims.sub, &new_display_name)?;
    }
    if let Some(new_password) = body.password {
        let password_hash = hash_password(&new_password)?;
        state.db.update_password(&claims.sub, &password_hash)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// POST /api/v1/user/apikey
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ApiKeyResponse {
    api_key: String,
}

async fn regenerate_apikey(
    AuthUser(claims): AuthUser,
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<ApiKeyResponse>> {
    let new_key = generate_api_key();
    let new_hash = hash_api_key(&new_key);

    state.db.update_api_key_hash(&claims.sub, &new_hash)?;

    Ok(Json(ApiKeyResponse { api_key: new_key }))
}
