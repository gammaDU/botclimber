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
    use argon2::{Argon2, PasswordHasher, password_hash::{SaltString, rand_core::OsRng}};

    let password_hash = {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(body.password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("argon2: {e}")))?
            .to_string()
    };

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

    let user_id: Uuid = row.id.parse().map_err(|_| AppError::Internal(anyhow::anyhow!("bad uuid")))?;
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
    user_id: String,
    username: String,
}

async fn profile(
    AuthUser(claims): AuthUser,
    State(_state): State<Arc<AppState>>,
) -> ApiResult<Json<ProfileResponse>> {
    Ok(Json(ProfileResponse {
        user_id: claims.sub,
        username: claims.username,
    }))
}

// ---------------------------------------------------------------------------
// PATCH /api/v1/user/profile
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct UpdateProfileRequest {
    display_name: String,
}

async fn update_profile(
    AuthUser(claims): AuthUser,
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateProfileRequest>,
) -> ApiResult<StatusCode> {
    state.db.update_display_name(&claims.sub, &body.display_name)?;
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
