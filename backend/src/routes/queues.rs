use crate::auth::rbac::{CanReadQueues, CanWriteQueues};
use crate::config_gen::queues as queues_gen;
use crate::models::queue::{PublicQueue, Queue, QueueInput};
use crate::AppState;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use std::path::PathBuf;
use uuid::Uuid;

fn generated_config_path() -> PathBuf {
    PathBuf::from("../asterisk-config/queues.conf")
}

/// Valid ring strategies per app_queue.so.
const VALID_STRATEGIES: &[&str] = &[
    "ringall",
    "leastrecent",
    "fewestcalls",
    "random",
    "rrmemory",
    "rrordered",
    "linear",
    "wrandom",
];

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/queues", get(list_queues).post(create_queue))
        .route(
            "/queues/:id",
            get(get_queue).put(update_queue).delete(delete_queue),
        )
        .route("/queues/reload", post(reload_asterisk))
}

// -----------------------------------------------------------------------------
// Handlers
// -----------------------------------------------------------------------------

async fn list_queues(
    State(state): State<AppState>,
    _perm: CanReadQueues,
) -> Result<Json<Vec<PublicQueue>>, StatusCode> {
    let rows: Vec<Queue> = sqlx::query_as("SELECT * FROM queues ORDER BY number")
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut out = Vec::with_capacity(rows.len());
    for q in rows {
        let members = members_for(&state, q.id).await?;
        out.push(public_queue(q, members));
    }
    Ok(Json(out))
}

async fn get_queue(
    State(state): State<AppState>,
    _perm: CanReadQueues,
    Path(id): Path<Uuid>,
) -> Result<Json<PublicQueue>, StatusCode> {
    let row: Queue = sqlx::query_as("SELECT * FROM queues WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let members = members_for(&state, row.id).await?;
    Ok(Json(public_queue(row, members)))
}

async fn create_queue(
    State(state): State<AppState>,
    _perm: CanWriteQueues,
    Json(input): Json<QueueInput>,
) -> Result<Json<PublicQueue>, StatusCode> {
    validate(&input)?;
    let member_ids = resolve_members(&state, &input.members).await?;

    let row: Queue = sqlx::query_as(
        "INSERT INTO queues
            (id, number, name, strategy, timeout, retry, maxlen, acw_seconds,
             musicclass, joinempty, leavewhenempty, ringinuse, active)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(&input.number)
    .bind(&input.name)
    .bind(&input.strategy)
    .bind(input.timeout)
    .bind(input.retry)
    .bind(input.maxlen)
    .bind(input.acw_seconds)
    .bind(&input.musicclass)
    .bind(input.joinempty)
    .bind(input.leavewhenempty)
    .bind(input.ringinuse)
    .bind(input.active)
    .fetch_one(&state.pool)
    .await
    .map_err(map_sqlx_error)?;

    replace_members(&state, row.id, &member_ids).await?;
    regenerate_and_reload_queues(&state).await?;

    let members = members_for(&state, row.id).await?;
    Ok(Json(public_queue(row, members)))
}

async fn update_queue(
    State(state): State<AppState>,
    _perm: CanWriteQueues,
    Path(id): Path<Uuid>,
    Json(input): Json<QueueInput>,
) -> Result<Json<PublicQueue>, StatusCode> {
    validate(&input)?;
    let member_ids = resolve_members(&state, &input.members).await?;

    let row: Queue = sqlx::query_as(
        "UPDATE queues SET
            number = $2, name = $3, strategy = $4, timeout = $5, retry = $6,
            maxlen = $7, acw_seconds = $8, musicclass = $9, joinempty = $10,
            leavewhenempty = $11, ringinuse = $12, active = $13,
            updated_at = now()
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(&input.number)
    .bind(&input.name)
    .bind(&input.strategy)
    .bind(input.timeout)
    .bind(input.retry)
    .bind(input.maxlen)
    .bind(input.acw_seconds)
    .bind(&input.musicclass)
    .bind(input.joinempty)
    .bind(input.leavewhenempty)
    .bind(input.ringinuse)
    .bind(input.active)
    .fetch_optional(&state.pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or(StatusCode::NOT_FOUND)?;

    replace_members(&state, row.id, &member_ids).await?;
    regenerate_and_reload_queues(&state).await?;

    let members = members_for(&state, row.id).await?;
    Ok(Json(public_queue(row, members)))
}

async fn delete_queue(
    State(state): State<AppState>,
    _perm: CanWriteQueues,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM queues WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    regenerate_and_reload_queues(&state).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn reload_asterisk(
    State(state): State<AppState>,
    _perm: CanWriteQueues,
) -> Result<Json<serde_json::Value>, StatusCode> {
    regenerate_and_reload_queues(&state).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

fn validate(input: &QueueInput) -> Result<(), StatusCode> {
    let n = &input.number;
    if n.len() != 4 || !n.starts_with('2') || !n.chars().all(|c| c.is_ascii_digit()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if !VALID_STRATEGIES.contains(&input.strategy.as_str()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.timeout < 1 || input.timeout > 300 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.retry < 1 || input.retry > 300 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.maxlen < 1 || input.maxlen > 1000 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.acw_seconds < 0 || input.acw_seconds > 600 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(())
}

fn map_sqlx_error(e: sqlx::Error) -> StatusCode {
    if let sqlx::Error::Database(db) = &e {
        if db.is_unique_violation() {
            return StatusCode::CONFLICT;
        }
        if db.is_foreign_key_violation() {
            return StatusCode::BAD_REQUEST;
        }
    }
    StatusCode::INTERNAL_SERVER_ERROR
}

fn public_queue(q: Queue, members: Vec<String>) -> PublicQueue {
    PublicQueue {
        id: q.id,
        number: q.number,
        name: q.name,
        strategy: q.strategy,
        timeout: q.timeout,
        retry: q.retry,
        maxlen: q.maxlen,
        acw_seconds: q.acw_seconds,
        musicclass: q.musicclass,
        joinempty: q.joinempty,
        leavewhenempty: q.leavewhenempty,
        ringinuse: q.ringinuse,
        active: q.active,
        members,
    }
}

async fn members_for(state: &AppState, queue_id: Uuid) -> Result<Vec<String>, StatusCode> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT e.number FROM queue_members qm
         JOIN extensions e ON e.id = qm.extension_id
         WHERE qm.queue_id = $1
         ORDER BY e.number",
    )
    .bind(queue_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(rows.into_iter().map(|(n,)| n).collect())
}

/// Resolve extension *numbers* to extension *IDs*. Returns an error if any
/// number doesn't exist.
async fn resolve_members(state: &AppState, numbers: &[String]) -> Result<Vec<Uuid>, StatusCode> {
    if numbers.is_empty() {
        return Ok(Vec::new());
    }
    let rows: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT id, number FROM extensions WHERE number = ANY($1)")
            .bind(numbers)
            .fetch_all(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows.len() != numbers.len() {
        // At least one number didn't resolve to an extension.
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(rows.into_iter().map(|(id, _)| id).collect())
}

async fn replace_members(
    state: &AppState,
    queue_id: Uuid,
    member_ids: &[Uuid],
) -> Result<(), StatusCode> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    sqlx::query("DELETE FROM queue_members WHERE queue_id = $1")
        .bind(queue_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for ext_id in member_ids {
        sqlx::query(
            "INSERT INTO queue_members (queue_id, extension_id, penalty)
             VALUES ($1, $2, 0)",
        )
        .bind(queue_id)
        .bind(ext_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    tx.commit()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

/// Regenerate queues.conf and reload queues.
///
/// Uses `safe_reload`: on reload failure the previous config is restored
/// and the reload is retried. See `config_gen::reload` for details.
async fn regenerate_and_reload_queues(state: &AppState) -> Result<(), StatusCode> {
    use crate::config_gen::reload::{safe_reload, ReloadCommand, ReloadOutcome};

    let rows: Vec<Queue> = sqlx::query_as("SELECT * FROM queues ORDER BY number")
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut member_lists: Vec<Vec<String>> = Vec::with_capacity(rows.len());
    for q in &rows {
        member_lists.push(members_for(state, q.id).await?);
    }

    let path = generated_config_path();
    let content = queues_gen::render(&rows, &member_lists);

    match safe_reload(state, &path, &content, ReloadCommand::Queues).await {
        Ok(ReloadOutcome::Applied) => Ok(()),
        Ok(ReloadOutcome::RolledBack { error }) => {
            tracing::error!("Queue reload failed, rolled back: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Ok(ReloadOutcome::Critical {
            reload_error,
            rollback_error,
        }) => {
            tracing::error!(
                "CRITICAL: Queue reload and rollback both failed. reload={} rollback={}",
                reload_error,
                rollback_error
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(e) => {
            tracing::error!("Queue reload: file write failed: {:#}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
