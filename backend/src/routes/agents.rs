use crate::auth::password;
use crate::auth::rbac::{CanReadAgents, CanWriteAgents};
use crate::config_gen::pjsip;
use crate::config_gen::queues as queues_gen;
use crate::config_gen::reload::{safe_reload, ReloadCommand, ReloadOutcome};
use crate::models::agent::{AgentInput, AgentUpdate, AgentView};
use crate::models::queue::Queue;
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde_json::json;
use std::path::PathBuf;
use uuid::Uuid;

fn pjsip_path() -> PathBuf {
    PathBuf::from("../asterisk-config/pjsip_extensions.conf")
}

fn queues_path() -> PathBuf {
    PathBuf::from("../asterisk-config/queues.conf")
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/agents", get(list_agents).post(create_agent))
        .route(
            "/agents/:id",
            get(get_agent).put(update_agent).delete(delete_agent),
        )
}

// ----------------------------------------------------------------------------
// List
// ----------------------------------------------------------------------------

async fn list_agents(
    State(state): State<AppState>,
    _perm: CanReadAgents,
) -> Result<Json<Vec<AgentView>>, StatusCode> {
    let rows: Vec<(Uuid, String, Option<String>, bool)> = sqlx::query_as(
        "SELECT id, username, extension, active FROM users
         WHERE role = 'agent'
         ORDER BY username",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut agents = Vec::with_capacity(rows.len());
    for (id, username, ext_num, active) in rows {
        agents.push(build_agent_view(&state, id, username, ext_num, active).await?);
    }
    Ok(Json(agents))
}

// ----------------------------------------------------------------------------
// Get one
// ----------------------------------------------------------------------------

async fn get_agent(
    State(state): State<AppState>,
    _perm: CanReadAgents,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentView>, StatusCode> {
    let row: Option<(Uuid, String, Option<String>, bool)> = sqlx::query_as(
        "SELECT id, username, extension, active FROM users
         WHERE id = $1 AND role = 'agent'",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (id, username, ext_num, active) = row.ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(build_agent_view(&state, id, username, ext_num, active).await?))
}

// ----------------------------------------------------------------------------
// Create
// ----------------------------------------------------------------------------

async fn create_agent(
    State(state): State<AppState>,
    _perm: CanWriteAgents,
    Json(input): Json<AgentInput>,
) -> Result<Json<AgentView>, StatusCode> {
    // Validate
    let n = &input.extension_number;
    if n.len() < 2 || n.len() > 10 || !n.chars().all(|c| c.is_ascii_digit()) {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.display_name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.sip_secret.len() < 6 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.password.len() < 6 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Check username / extension not taken
    let exists: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM users WHERE username = $1")
            .bind(n)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if exists.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let exists: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM extensions WHERE number = $1")
            .bind(n)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if exists.is_some() {
        return Err(StatusCode::CONFLICT);
    }

    let password_hash = password::hash_password(&input.password)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_id = Uuid::new_v4();
    let extension_id = Uuid::new_v4();

    // Insert user
    sqlx::query(
        "INSERT INTO users (id, username, password_hash, role, extension, active)
         VALUES ($1, $2, $3, 'agent', $4, $5)",
    )
    .bind(user_id)
    .bind(n)
    .bind(&password_hash)
    .bind(n)
    .bind(input.active)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Insert extension
    sqlx::query(
        "INSERT INTO extensions
            (id, number, display_name, secret, context, transport, codecs, max_contacts, active)
         VALUES ($1, $2, $3, $4, 'from-internal', 'transport-udp',
                 ARRAY['ulaw','alaw']::TEXT[], 1, $5)",
    )
    .bind(extension_id)
    .bind(n)
    .bind(&input.display_name)
    .bind(&input.sip_secret)
    .bind(input.active)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Assign to queues
    for qnum in &input.queue_numbers {
        let q: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM queues WHERE number = $1")
            .bind(qnum)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if let Some((queue_id,)) = q {
            let _ = sqlx::query(
                "INSERT INTO queue_members (queue_id, extension_id, penalty)
                 VALUES ($1, $2, 0) ON CONFLICT DO NOTHING",
            )
            .bind(queue_id)
            .bind(extension_id)
            .execute(&state.pool)
            .await;
        }
    }

    regenerate_and_reload(&state).await?;
    Ok(Json(build_agent_view(&state, user_id, n.clone(), Some(n.clone()), input.active).await?))
}

// ----------------------------------------------------------------------------
// Update
// ----------------------------------------------------------------------------

async fn update_agent(
    State(state): State<AppState>,
    _perm: CanWriteAgents,
    Path(id): Path<Uuid>,
    Json(input): Json<AgentUpdate>,
) -> Result<Json<AgentView>, StatusCode> {
    let row: Option<(String, Option<String>, bool)> = sqlx::query_as(
        "SELECT username, extension, active FROM users
         WHERE id = $1 AND role = 'agent'",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (username, ext_opt, current_active) = row.ok_or(StatusCode::NOT_FOUND)?;
    let ext_number = ext_opt.ok_or(StatusCode::BAD_REQUEST)?;

    let new_active = input.active.unwrap_or(current_active);

    // Update user (password if provided, else leave)
    if let Some(pw) = &input.password {
        if pw.len() < 6 {
            return Err(StatusCode::BAD_REQUEST);
        }
        let hash = password::hash_password(pw).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        sqlx::query("UPDATE users SET password_hash = $1, active = $2 WHERE id = $3")
            .bind(&hash)
            .bind(new_active)
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        sqlx::query("UPDATE users SET active = $1 WHERE id = $2")
            .bind(new_active)
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // Update extension fields
    if let Some(dn) = &input.display_name {
        if dn.trim().is_empty() {
            return Err(StatusCode::BAD_REQUEST);
        }
        sqlx::query("UPDATE extensions SET display_name = $1 WHERE number = $2")
            .bind(dn)
            .bind(&ext_number)
            .execute(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    if let Some(ss) = &input.sip_secret {
        if ss.len() < 6 {
            return Err(StatusCode::BAD_REQUEST);
        }
        sqlx::query("UPDATE extensions SET secret = $1 WHERE number = $2")
            .bind(ss)
            .bind(&ext_number)
            .execute(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    // Reflect active flag on extension
    sqlx::query("UPDATE extensions SET active = $1 WHERE number = $2")
        .bind(new_active)
        .bind(&ext_number)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Update queue memberships
    if let Some(queue_numbers) = &input.queue_numbers {
        let ext_id: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM extensions WHERE number = $1")
                .bind(&ext_number)
                .fetch_optional(&state.pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let ext_id = ext_id.map(|(id,)| id).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        sqlx::query("DELETE FROM queue_members WHERE extension_id = $1")
            .bind(ext_id)
            .execute(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        for qnum in queue_numbers {
            let q: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM queues WHERE number = $1")
                .bind(qnum)
                .fetch_optional(&state.pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if let Some((queue_id,)) = q {
                let _ = sqlx::query(
                    "INSERT INTO queue_members (queue_id, extension_id, penalty)
                     VALUES ($1, $2, 0) ON CONFLICT DO NOTHING",
                )
                .bind(queue_id)
                .bind(ext_id)
                .execute(&state.pool)
                .await;
            }
        }
    }

    regenerate_and_reload(&state).await?;
    Ok(Json(build_agent_view(&state, id, username, Some(ext_number), new_active).await?))
}

// ----------------------------------------------------------------------------
// Delete (soft)
// ----------------------------------------------------------------------------

async fn delete_agent(
    State(state): State<AppState>,
    _perm: CanWriteAgents,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let row: Option<(String, Option<String>)> = sqlx::query_as(
        "SELECT username, extension FROM users WHERE id = $1 AND role = 'agent'",
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (_username, ext_opt) = row.ok_or(StatusCode::NOT_FOUND)?;

    // Soft-delete: deactivate the user and the extension. Keep rows for
    // audit history (CDRs, call notes, agent_status_log reference them).
    sqlx::query("UPDATE users SET active = false WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(ext_number) = ext_opt {
        sqlx::query("UPDATE extensions SET active = false WHERE number = $1")
            .bind(&ext_number)
            .execute(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Remove from all queues.
        let ext_id: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM extensions WHERE number = $1")
                .bind(&ext_number)
                .fetch_optional(&state.pool)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if let Some((eid,)) = ext_id {
            let _ = sqlx::query("DELETE FROM queue_members WHERE extension_id = $1")
                .bind(eid)
                .execute(&state.pool)
                .await;
        }
    }

    regenerate_and_reload(&state).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ----------------------------------------------------------------------------
// Helpers
// ----------------------------------------------------------------------------

async fn build_agent_view(
    state: &AppState,
    user_id: Uuid,
    username: String,
    extension_number: Option<String>,
    active: bool,
) -> Result<AgentView, StatusCode> {
    let ext_number = extension_number.clone();

    let (display_name, sip_secret) = if let Some(ref num) = ext_number {
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT display_name, secret FROM extensions WHERE number = $1",
        )
        .bind(num)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        match row {
            Some((dn, s)) => (Some(dn), Some(s)),
            None => (None, None),
        }
    } else {
        (None, None)
    };

    let queue_numbers: Vec<String> = if let Some(ref num) = ext_number {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT q.number FROM queue_members qm
             JOIN queues q ON q.id = qm.queue_id
             JOIN extensions e ON e.id = qm.extension_id
             WHERE e.number = $1
             ORDER BY q.number",
        )
        .bind(num)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        rows.into_iter().map(|(n,)| n).collect()
    } else {
        Vec::new()
    };

    Ok(AgentView {
        id: user_id,
        username,
        extension_number: ext_number,
        display_name,
        sip_secret,
        queue_numbers,
        active,
    })
}

/// Regenerate pjsip + queues configs and reload both.
async fn regenerate_and_reload(state: &AppState) -> Result<(), StatusCode> {
    // PJSIP
    let exts: Vec<crate::models::extension::Extension> =
        sqlx::query_as("SELECT * FROM extensions ORDER BY number")
            .fetch_all(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let pjsip_content = pjsip::render(&exts);
    match safe_reload(
        state,
        &pjsip_path(),
        &pjsip_content,
        ReloadCommand::Pjsip,
    )
    .await
    {
        Ok(ReloadOutcome::Applied) => {}
        other => {
            tracing::error!("PJSIP reload after agent change: {:?}", other);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Queues
    let queues: Vec<Queue> = sqlx::query_as("SELECT * FROM queues ORDER BY number")
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut member_lists: Vec<Vec<String>> = Vec::with_capacity(queues.len());
    for q in &queues {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT e.number FROM queue_members qm
             JOIN extensions e ON e.id = qm.extension_id
             WHERE qm.queue_id = $1 AND e.active = true
             ORDER BY e.number",
        )
        .bind(q.id)
        .fetch_all(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        member_lists.push(rows.into_iter().map(|(n,)| n).collect());
    }

    let queues_content = queues_gen::render(&queues, &member_lists);
    match safe_reload(
        state,
        &queues_path(),
        &queues_content,
        ReloadCommand::Queues,
    )
    .await
    {
        Ok(ReloadOutcome::Applied) => Ok(()),
        other => {
            tracing::error!("Queue reload after agent change: {:?}", other);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Silence unused import warning when this module is compiled standalone.
#[allow(dead_code)]
fn _keep(_: serde_json::Value) {
    let _ = json!({});
}
