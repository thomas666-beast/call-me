use crate::auth::rbac::{CanReadConfig, CanWriteConfig};
use crate::config_gen::pjsip;
use crate::config_gen::reload::{safe_reload, ReloadCommand, ReloadOutcome};
use crate::models::extension::{Extension, ExtensionInput, PublicExtension};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::path::PathBuf;
use uuid::Uuid;

/// Path (on the host, visible to the container via bind mount) where the
/// generated PJSIP extensions file lives.
fn generated_config_path() -> PathBuf {
    PathBuf::from("../asterisk-config/pjsip_extensions.conf")
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/extensions", get(list_extensions).post(create_extension))
        .route(
            "/extensions/:id",
            get(get_extension).put(update_extension).delete(delete_extension),
        )
        .route("/extensions/reload", post(reload_asterisk))
}

// -----------------------------------------------------------------------------
// Handlers
// -----------------------------------------------------------------------------

async fn list_extensions(
    State(state): State<AppState>,
    _perm: CanReadConfig,
) -> Result<Json<Vec<PublicExtension>>, StatusCode> {
    let rows: Vec<Extension> =
        sqlx::query_as("SELECT * FROM extensions ORDER BY number")
            .fetch_all(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rows.into_iter().map(Into::into).collect()))
}

async fn get_extension(
    State(state): State<AppState>,
    _perm: CanReadConfig,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicExtension>, StatusCode> {
    let row: Extension = sqlx::query_as("SELECT * FROM extensions WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(row.into()))
}

async fn create_extension(
    State(state): State<AppState>,
    _perm: CanWriteConfig,
    Json(input): Json<ExtensionInput>,
) -> Result<Json<PublicExtension>, StatusCode> {
    validate(&input)?;

    let row: Extension = sqlx::query_as(
        "INSERT INTO extensions
            (id, number, display_name, secret, context, transport, codecs, max_contacts, active)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(&input.number)
    .bind(&input.display_name)
    .bind(&input.secret)
    .bind(&input.context)
    .bind(&input.transport)
    .bind(&input.codecs)
    .bind(input.max_contacts)
    .bind(input.active)
    .fetch_one(&state.pool)
    .await
    .map_err(map_sqlx_error)?;

    regenerate_and_reload_pjsip(&state).await?;
    Ok(Json(row.into()))
}

async fn update_extension(
    State(state): State<AppState>,
    _perm: CanWriteConfig,
    Path(id): Path<Uuid>,
    Json(input): Json<ExtensionInput>,
) -> Result<Json<PublicExtension>, StatusCode> {
    validate(&input)?;

    let row: Extension = sqlx::query_as(
        "UPDATE extensions SET
            number = $2,
            display_name = $3,
            secret = $4,
            context = $5,
            transport = $6,
            codecs = $7,
            max_contacts = $8,
            active = $9,
            updated_at = now()
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(&input.number)
    .bind(&input.display_name)
    .bind(&input.secret)
    .bind(&input.context)
    .bind(&input.transport)
    .bind(&input.codecs)
    .bind(input.max_contacts)
    .bind(input.active)
    .fetch_optional(&state.pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or(StatusCode::NOT_FOUND)?;

    regenerate_and_reload_pjsip(&state).await?;
    Ok(Json(row.into()))
}

async fn delete_extension(
    State(state): State<AppState>,
    _perm: CanWriteConfig,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM extensions WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    regenerate_and_reload_pjsip(&state).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn reload_asterisk(
    State(state): State<AppState>,
    _perm: CanWriteConfig,
) -> Result<Json<serde_json::Value>, StatusCode> {
    regenerate_and_reload_pjsip(&state).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn validate(input: &ExtensionInput) -> Result<(), StatusCode> {
    let n = &input.number;
    if n.len() < 2 || n.len() > 10 || !n.chars().all(|c| c.is_ascii_digit()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.display_name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.secret.len() < 6 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.max_contacts < 1 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

fn map_sqlx_error(e: sqlx::Error) -> StatusCode {
    if let sqlx::Error::Database(db) = &e {
        if db.is_unique_violation() {
            return StatusCode::CONFLICT;
        }
    }
    StatusCode::INTERNAL_SERVER_ERROR
}

/// Regenerate pjsip_extensions.conf and reload PJSIP.
///
/// Uses `safe_reload`: on reload failure the previous config is restored
/// and the reload is retried. See `config_gen::reload` for details.
async fn regenerate_and_reload_pjsip(state: &AppState) -> Result<(), StatusCode> {
    let rows: Vec<Extension> = sqlx::query_as("SELECT * FROM extensions ORDER BY number")
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let path = generated_config_path();
    let content = pjsip::render(&rows);

    match safe_reload(state, &path, &content, ReloadCommand::Pjsip).await {
        Ok(ReloadOutcome::Applied) => Ok(()),
        Ok(ReloadOutcome::RolledBack { error }) => {
            tracing::error!("PJSIP reload failed, rolled back: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(ReloadOutcome::Critical {
            reload_error,
            rollback_error,
        }) => {
            tracing::error!(
                "CRITICAL: PJSIP reload and rollback both failed. reload={} rollback={}",
                reload_error,
                rollback_error
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(e) => {
            tracing::error!("PJSIP reload: file write failed: {:#}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
