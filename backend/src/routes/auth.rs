use crate::auth::{jwt, middleware::AuthUser, password};
use crate::models::user::{PublicUser, User};
use crate::AppState;
use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::{Duration, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::SocketAddr;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub user: PublicUser,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Serialize)]
pub struct RefreshResponse {
    pub access_token: String,
}

#[derive(Deserialize)]
pub struct LogoutRequest {
    pub refresh_token: String,
}

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
}

pub fn protected_router() -> Router<AppState> {
    Router::new().route("/me", get(me))
}

async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let ip = addr.ip();

    // Rate limit check. Reject before touching the DB.
    if let Err(retry_after) = state.rate_limiter.check_and_record(ip).await {
        tracing::warn!(
            "Login rate limit hit for IP {} (retry after {:?})",
            ip,
            retry_after
        );
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    let user: Option<User> =
        sqlx::query_as("SELECT * FROM users WHERE username = $1 AND active = true")
            .bind(&req.username)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ok = user
        .as_ref()
        .map(|u| password::verify_password(&req.password, &u.password_hash))
        .unwrap_or(false);

    let _ = sqlx::query("INSERT INTO auth_audit_log (username, success) VALUES ($1, $2)")
        .bind(&req.username)
        .bind(ok)
        .execute(&state.pool)
        .await;

    let user = user.ok_or(StatusCode::UNAUTHORIZED)?;
    if !ok {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Successful login — clear the rate limit for this IP.
    state.rate_limiter.clear(ip).await;

    let access = jwt::issue_access_token(
        &state.cfg.jwt_secret,
        user.id,
        &user.username,
        &user.role,
        state.cfg.access_token_ttl_minutes,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let refresh = generate_refresh_token();
    let refresh_hash = hash_token(&refresh);
    let expires_at = Utc::now() + Duration::days(state.cfg.refresh_token_ttl_days);

    sqlx::query(
        "INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::new_v4())
    .bind(user.id)
    .bind(&refresh_hash)
    .bind(expires_at)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse {
        access_token: access,
        refresh_token: refresh,
        user: user.into(),
    }))
}

async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, StatusCode> {
    let hash = hash_token(&req.refresh_token);

    let row: Option<(Uuid, Uuid, chrono::DateTime<Utc>, bool)> = sqlx::query_as(
        "SELECT id, user_id, expires_at, revoked FROM refresh_tokens WHERE token_hash = $1",
    )
    .bind(&hash)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (_, user_id, expires_at, revoked) = row.ok_or(StatusCode::UNAUTHORIZED)?;
    if revoked || expires_at < Utc::now() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1 AND active = true")
        .bind(user_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let access = jwt::issue_access_token(
        &state.cfg.jwt_secret,
        user.id,
        &user.username,
        &user.role,
        state.cfg.access_token_ttl_minutes,
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(RefreshResponse { access_token: access }))
}

async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> Result<StatusCode, StatusCode> {
    let hash = hash_token(&req.refresh_token);
    sqlx::query("UPDATE refresh_tokens SET revoked = true WHERE token_hash = $1")
        .bind(&hash)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn me(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> Result<Json<PublicUser>, StatusCode> {
    let user_id: Uuid = claims.sub.parse().map_err(|_| StatusCode::UNAUTHORIZED)?;

    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1 AND active = true")
        .bind(user_id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(Json(user.into()))
}

fn generate_refresh_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}
