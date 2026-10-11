use crate::auth::password;
use crate::auth::rbac::{CanReadUsers, CanWriteUsers};
use crate::models::user::{PublicUser, User, UserInput, UserUpdate};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use uuid::Uuid;

/// Roles that can log into the admin panel. Agents are managed through
/// the Agents menu because they also need an extension and queues.
const PANEL_ROLES: &[&str] = &["admin", "supervisor"];

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
}

// -----------------------------------------------------------------------------
// List
// -----------------------------------------------------------------------------

async fn list_users(
    State(state): State<AppState>,
    _perm: CanReadUsers,
) -> Result<Json<Vec<PublicUser>>, StatusCode> {
    let users: Vec<User> = sqlx::query_as(
        "SELECT * FROM users
         WHERE role IN ('admin', 'supervisor')
         ORDER BY username",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(users.into_iter().map(Into::into).collect()))
}

async fn get_user(
    State(state): State<AppState>,
    _perm: CanReadUsers,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicUser>, StatusCode> {
    let user: User = sqlx::query_as(
        "SELECT * FROM users
         WHERE id = $1 AND role IN ('admin', 'supervisor')",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(user.into()))
}

// -----------------------------------------------------------------------------
// Create
// -----------------------------------------------------------------------------

async fn create_user(
    State(state): State<AppState>,
    _perm: CanWriteUsers,
    Json(input): Json<UserInput>,
) -> Result<Json<PublicUser>, StatusCode> {
    validate_username(&input.username)?;
    validate_password(&input.password)?;
    if !PANEL_ROLES.contains(&input.role.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Username must be unique across all users (including agents).
    let exists: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM users WHERE username = $1")
            .bind(&input.username)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if exists.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let hash = password::hash_password(&input.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let email = input.email.as_ref().filter(|e| !e.trim().is_empty());

    let user: User = sqlx::query_as(
        "INSERT INTO users
            (id, username, email, password_hash, role, extension, active)
         VALUES ($1, $2, $3, $4, $5, NULL, $6)
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(&input.username)
    .bind(email)
    .bind(&hash)
    .bind(&input.role)
    .bind(input.active)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user.into()))
}

// -----------------------------------------------------------------------------
// Update
// -----------------------------------------------------------------------------

async fn update_user(
    State(state): State<AppState>,
    _perm: CanWriteUsers,
    Path(id): Path<Uuid>,
    Json(input): Json<UserUpdate>,
) -> Result<Json<PublicUser>, StatusCode> {
    let current: Option<(Option<String>, String, bool)> = sqlx::query_as(
        "SELECT email, role, active FROM users
         WHERE id = $1 AND role IN ('admin', 'supervisor')",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let (current_email, _role, current_active) = current.ok_or(StatusCode::NOT_FOUND)?;

    let new_active = input.active.unwrap_or(current_active);

    // Refuse to deactivate the last active admin.
    if current_active && !new_active {
        let (remaining,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users
             WHERE role = 'admin' AND active = true AND id <> $1",
        )
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if remaining == 0 {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let email = input
        .email
        .as_ref()
        .map(|e| if e.trim().is_empty() { None } else { Some(e.clone()) })
        .unwrap_or(current_email);

    if let Some(pw) = &input.password {
        if !pw.is_empty() {
            validate_password(pw)?;
            let hash = password::hash_password(pw)
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            sqlx::query(
                "UPDATE users
                 SET email = $1, password_hash = $2, active = $3, updated_at = now()
                 WHERE id = $4",
            )
            .bind(&email)
            .bind(&hash)
            .bind(new_active)
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    if input.password.as_ref().map(|p| p.is_empty()).unwrap_or(true) {
        sqlx::query(
            "UPDATE users SET email = $1, active = $2, updated_at = now()
             WHERE id = $3",
        )
        .bind(&email)
        .bind(new_active)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(user.into()))
}

// -----------------------------------------------------------------------------
// Delete
// -----------------------------------------------------------------------------

async fn delete_user(
    State(state): State<AppState>,
    _perm: CanWriteUsers,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let current: Option<(String, bool)> = sqlx::query_as(
        "SELECT role, active FROM users
         WHERE id = $1 AND role IN ('admin', 'supervisor')",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let (role, was_active) = current.ok_or(StatusCode::NOT_FOUND)?;

    // Refuse to deactivate the last active admin.
    if role == "admin" && was_active {
        let (remaining,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM users
             WHERE role = 'admin' AND active = true AND id <> $1",
        )
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if remaining == 0 {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    sqlx::query("UPDATE users SET active = false, updated_at = now() WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

// -----------------------------------------------------------------------------
// Validation
// -----------------------------------------------------------------------------

fn validate_username(username: &str) -> Result<(), StatusCode> {
    let u = username.trim();
    if u.len() < 3 || u.len() > 32 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !u.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-') {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), StatusCode> {
    if password.len() < 8 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}
