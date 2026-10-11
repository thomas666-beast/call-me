use crate::ami;
use crate::auth::jwt::Claims;
use crate::auth::middleware::AuthUser;
use crate::auth::rbac::CanReadContacts;
use crate::models::agent_status::is_valid_reason;
use crate::models::call_note::{CallNoteInput, PublicCallNote};
use crate::models::cdr::Cdr;
use crate::models::contact::{
    normalize_phone, Contact, ContactInput, ContactQuery, ContactWithCalls,
};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me))
        .route("/me/queues", get(list_my_queues))
        .route("/queues", get(list_all_queues))
        .route("/extensions", get(list_all_extensions))
        .route("/me/queues/:queue/pause", post(pause_in_queue))
        .route("/me/queues/:queue/unpause", post(unpause_in_queue))
        .route("/me/calls", get(list_my_calls))
        .route("/me/active", get(get_active_call))
        .route(
            "/me/calls/:uniqueid/notes",
            get(list_call_notes).post(create_call_note),
        )
        .route("/contacts", get(list_contacts).post(create_contact))
        .route("/contacts/lookup/:number", get(lookup_contact))
        .route("/contacts/:id", get(get_contact))
}

// -----------------------------------------------------------------------------
// Response types
// -----------------------------------------------------------------------------

#[derive(Serialize)]
pub struct AgentMe {
    pub username: String,
    pub role: String,
    pub extension: Option<String>,
}

#[derive(Serialize)]
pub struct AgentQueue {
    pub number: String,
    pub name: String,
    pub paused: bool,
    pub pause_reason: Option<String>,
}

#[derive(Serialize)]
pub struct ActiveCall {
    pub callid: String,
    pub linkedid: Option<String>,
    pub queue_number: String,
    pub caller_number: String,
    pub connected_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct PauseBody {
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(FromRow)]
struct NoteRow {
    id: Uuid,
    call_uniqueid: String,
    call_linkedid: String,
    author_id: Uuid,
    author_username: String,
    note: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct ActiveCallRow {
    callid: String,
    linkedid: Option<String>,
    queuename: String,
    time: DateTime<Utc>,
    caller_number: String,
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

async fn user_extension(state: &AppState, claims: &Claims) -> Result<Option<String>, StatusCode> {
    let user_id: Uuid = claims.sub.parse().map_err(|_| StatusCode::UNAUTHORIZED)?;
    let row: Option<(Option<String>,)> =
        sqlx::query_as("SELECT extension FROM users WHERE id = $1 AND active = true")
            .bind(user_id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(row.and_then(|(e,)| e))
}

async fn require_extension(state: &AppState, claims: &Claims) -> Result<String, StatusCode> {
    user_extension(state, claims)
        .await?
        .ok_or(StatusCode::BAD_REQUEST)
}

// -----------------------------------------------------------------------------
// GET /api/agent/me
// -----------------------------------------------------------------------------

async fn get_me(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> Result<Json<AgentMe>, StatusCode> {
    let ext = user_extension(&state, &claims).await?;
    Ok(Json(AgentMe {
        username: claims.username.clone(),
        role: claims.role.clone(),
        extension: ext,
    }))
}

// -----------------------------------------------------------------------------
// GET /api/agent/me/queues
// -----------------------------------------------------------------------------

async fn list_my_queues(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> Result<Json<Vec<AgentQueue>>, StatusCode> {
    let ext = require_extension(&state, &claims).await?;

    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT q.number, q.name
         FROM queue_members qm
         JOIN queues q ON q.id = qm.queue_id
         JOIN extensions e ON e.id = qm.extension_id
         WHERE e.number = $1 AND q.active = true
         ORDER BY q.number",
    )
    .bind(&ext)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let members = ami::actions::queue_status(&state.cfg)
        .await
        .unwrap_or_default();

    let my_interface = format!("PJSIP/{}", ext);

    let mut result = Vec::with_capacity(rows.len());
    for (number, name) in rows {
        // Find this agent's membership in this queue from the AMI snapshot.
        let membership = members
            .iter()
            .find(|m| m.interface == my_interface && m.queue == number);

        let paused = membership.map(|m| m.paused).unwrap_or(false);

        // Prefer Asterisk's reason; fall back to the DB log if Asterisk
        // didn't send one.
        let pause_reason = if paused {
            match membership.and_then(|m| m.paused_reason.clone()) {
                Some(r) => Some(r),
                None => {
                    let reason_row: Option<(Option<String>,)> = sqlx::query_as(
                        "SELECT reason FROM agent_status_log
                         WHERE extension = $1 AND queue_number = $2
                         ORDER BY occurred_at DESC
                         LIMIT 1",
                    )
                    .bind(&ext)
                    .bind(&number)
                    .fetch_optional(&state.pool)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                    reason_row.and_then(|(r,)| r)
                }
            }
        } else {
            None
        };

        result.push(AgentQueue {
            number,
            name,
            paused,
            pause_reason,
        });
    }

    Ok(Json(result))
}

/// Minimal list of all queues (number + name). Used by the frontend
/// directory so agents can resolve queue numbers to names in their
/// call history. No sensitive data — just what any employee would
/// see on a phone display.
#[derive(serde::Serialize)]
pub struct QueueLookup {
    pub number: String,
    pub name: String,
}

async fn list_all_queues(
    State(state): State<AppState>,
    Extension(AuthUser(_claims)): Extension<AuthUser>,
) -> Result<Json<Vec<QueueLookup>>, StatusCode> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT number, name FROM queues WHERE active = true ORDER BY number",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        rows.into_iter()
            .map(|(number, name)| QueueLookup { number, name })
            .collect(),
    ))
}

// -----------------------------------------------------------------------------
// POST /api/agent/me/queues/:queue/pause
// POST /api/agent/me/queues/:queue/unpause
// -----------------------------------------------------------------------------

async fn pause_in_queue(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Path(queue): Path<String>,
    Json(body): Json<PauseBody>,
) -> Result<Json<AgentQueue>, StatusCode> {
    let ext = require_extension(&state, &claims).await?;
    let reason = body.reason.unwrap_or_else(|| "other".to_string());
    if !is_valid_reason(&reason) {
        return Err(StatusCode::BAD_REQUEST);
    }
    set_pause(&state, &claims, &ext, &queue, true, Some(&reason)).await
}

async fn unpause_in_queue(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Path(queue): Path<String>,
) -> Result<Json<AgentQueue>, StatusCode> {
    let ext = require_extension(&state, &claims).await?;
    set_pause(&state, &claims, &ext, &queue, false, None).await
}

async fn set_pause(
    state: &AppState,
    claims: &Claims,
    ext: &str,
    queue: &str,
    paused: bool,
    reason: Option<&str>,
) -> Result<Json<AgentQueue>, StatusCode> {
    let member: Option<(String,)> = sqlx::query_as(
        "SELECT q.name
         FROM queue_members qm
         JOIN queues q ON q.id = qm.queue_id
         JOIN extensions e ON e.id = qm.extension_id
         WHERE e.number = $1 AND q.number = $2",
    )
    .bind(ext)
    .bind(queue)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (queue_name,) = member.ok_or(StatusCode::NOT_FOUND)?;
    let interface = format!("PJSIP/{}", ext);

    ami::actions::queue_pause(&state.cfg, queue, &interface, paused, reason)
        .await
        .map_err(|e| {
            tracing::error!("QueuePause failed: {:#}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let actor_id: Option<Uuid> = claims.sub.parse().ok();
    sqlx::query(
        "INSERT INTO agent_status_log
            (extension, queue_number, event_type, reason, actor_id)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(ext)
    .bind(queue)
    .bind(if paused { "pause" } else { "unpause" })
    .bind(reason)
    .bind(actor_id)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AgentQueue {
        number: queue.to_string(),
        name: queue_name,
        paused,
        pause_reason: reason.map(|s| s.to_string()),
    }))
}

// -----------------------------------------------------------------------------
// GET /api/agent/me/calls
// -----------------------------------------------------------------------------

async fn list_my_calls(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Cdr>>, StatusCode> {
    let ext = require_extension(&state, &claims).await?;
    let limit = q.limit.clamp(1, 500);
    let offset = q.offset.max(0);
    let chan_pattern = format!("PJSIP/{}-%", ext);

    let rows: Vec<Cdr> = sqlx::query_as(
        "SELECT * FROM (
            SELECT DISTINCT ON (COALESCE(NULLIF(c.linkedid, ''), c.uniqueid))
                c.id, c.calldate, c.start_time, c.answer_time, c.end_time,
                c.clid, c.src, c.dst, c.dcontext, c.channel, c.dstchannel,
                c.lastapp, c.lastdata, c.duration, c.billsec, c.disposition,
                c.amaflags, c.accountcode, c.uniqueid, c.userfield,
                c.peeraccount, c.linkedid, c.sequence,

                CASE
                    WHEN EXISTS (SELECT 1 FROM queues q WHERE q.number = c.dst)
                        THEN 'queue'
                    WHEN EXISTS (SELECT 1 FROM extensions e WHERE e.number = c.src)
                     AND EXISTS (SELECT 1 FROM extensions e WHERE e.number = c.dst)
                        THEN 'internal'
                    WHEN EXISTS (SELECT 1 FROM extensions e WHERE e.number = c.src)
                        THEN 'outbound'
                    ELSE 'external'
                END AS kind
            FROM cdr c
            WHERE (c.src = $1 OR c.dst = $1
                   OR c.dstchannel LIKE $2 OR c.channel LIKE $2)
              AND c.disposition = 'ANSWERED'
            ORDER BY
                COALESCE(NULLIF(c.linkedid, ''), c.uniqueid),
                -- Prefer the leg where src is NOT an extension number.
                -- For queue calls, that's the caller side (src = external
                -- number, dst = queue). For internal calls, both legs have
                -- extension srcs and this tiebreak does nothing, which is
                -- fine — either leg works.
                CASE WHEN EXISTS (
                    SELECT 1 FROM extensions e WHERE e.number = c.src
                ) THEN 1 ELSE 0 END ASC,
                c.calldate DESC,
                c.id DESC
        ) sub
        ORDER BY calldate DESC, id DESC
        LIMIT $3 OFFSET $4",
    )
    .bind(&ext)
    .bind(&chan_pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("agent call list failed: {:#}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(rows))
}

// -----------------------------------------------------------------------------
// GET /api/agent/me/active
// -----------------------------------------------------------------------------

async fn get_active_call(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
) -> Result<Json<Option<ActiveCall>>, StatusCode> {
    let ext = require_extension(&state, &claims).await?;
    let interface = format!("PJSIP/{}", ext);

    // During an active call, CDR isn't written yet. Get the caller number
    // from CEL, which is written live as channel events happen.
    let row: Option<ActiveCallRow> = sqlx::query_as(
        "SELECT
            ql.callid,
            (SELECT linkedid FROM cdr WHERE uniqueid = ql.callid OR linkedid = ql.callid LIMIT 1) AS linkedid,
            ql.queuename,
            ql.\"time\" AS time,
            COALESCE(
                (SELECT cid_num FROM cel
                 WHERE (uniqueid = ql.callid OR linkedid = ql.callid)
                   AND cid_num <> ''
                 ORDER BY eventtime ASC
                 LIMIT 1),
                ''
            ) AS caller_number
         FROM queue_log ql
         WHERE ql.agent = $1
           AND ql.event = 'CONNECT'
           AND ql.\"time\" > now() - interval '6 hours'
           AND NOT EXISTS (
               SELECT 1 FROM queue_log ql2
               WHERE ql2.callid = ql.callid
                 AND ql2.event IN ('COMPLETECALLER','COMPLETEAGENT','ABANDON')
                 AND ql2.\"time\" > ql.\"time\"
           )
         ORDER BY ql.\"time\" DESC
         LIMIT 1",
    )
    .bind(&interface)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("active call query failed: {:#}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(row.map(|r| ActiveCall {
        callid: r.callid,
        linkedid: r.linkedid,
        queue_number: r.queuename,
        caller_number: r.caller_number,
        connected_at: r.time,
    })))
}

// -----------------------------------------------------------------------------
// Call notes
// -----------------------------------------------------------------------------

async fn list_call_notes(
    State(state): State<AppState>,
    Extension(AuthUser(_claims)): Extension<AuthUser>,
    Path(uniqueid): Path<String>,
) -> Result<Json<Vec<PublicCallNote>>, StatusCode> {
    let rows: Vec<NoteRow> = sqlx::query_as(
        "SELECT n.id, n.call_uniqueid, n.call_linkedid, n.author_id,
                u.username AS author_username, n.note, n.created_at, n.updated_at
         FROM call_notes n
         JOIN users u ON u.id = n.author_id
         WHERE n.call_uniqueid = $1 OR n.call_linkedid = $1
         ORDER BY n.created_at ASC",
    )
    .bind(&uniqueid)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let notes = rows
        .into_iter()
        .map(|r| PublicCallNote {
            id: r.id,
            call_uniqueid: r.call_uniqueid,
            call_linkedid: r.call_linkedid,
            author_id: r.author_id,
            author_username: r.author_username,
            note: r.note,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
        .collect();

    Ok(Json(notes))
}

async fn create_call_note(
    State(state): State<AppState>,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Path(uniqueid): Path<String>,
    Json(input): Json<CallNoteInput>,
) -> Result<Json<PublicCallNote>, StatusCode> {
    let note = input.note.trim();
    if note.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let author_id: Uuid = claims.sub.parse().map_err(|_| StatusCode::UNAUTHORIZED)?;

    let linkedid_row: Option<(String,)> = sqlx::query_as(
        "SELECT linkedid FROM cdr
         WHERE uniqueid = $1 OR linkedid = $1
         LIMIT 1",
    )
    .bind(&uniqueid)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let linkedid = linkedid_row.map(|(l,)| l).unwrap_or_default();

    let id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO call_notes
            (id, call_uniqueid, call_linkedid, author_id, note)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(&uniqueid)
    .bind(&linkedid)
    .bind(author_id)
    .bind(note)
    .execute(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PublicCallNote {
        id,
        call_uniqueid: uniqueid.clone(),
        call_linkedid: linkedid,
        author_id,
        author_username: claims.username.clone(),
        note: note.to_string(),
        created_at: now,
        updated_at: now,
    }))
}

// -----------------------------------------------------------------------------
// Contacts (agent-scoped, read + create only)
// -----------------------------------------------------------------------------

async fn list_contacts(
    State(state): State<AppState>,
    _perm: CanReadContacts,
    Query(q): Query<ContactQuery>,
) -> Result<Json<Vec<Contact>>, StatusCode> {
    let limit = q.limit.clamp(1, 500);
    let offset = q.offset.max(0);

    let rows: Vec<Contact> = sqlx::query_as(
        "SELECT * FROM contacts
         WHERE ($1::text IS NULL
                OR display_name ILIKE '%' || $1 || '%'
                OR company      ILIKE '%' || $1 || '%'
                OR notes        ILIKE '%' || $1 || '%'
                OR phone_number ILIKE '%' || $1 || '%'
                OR (
                    regexp_replace($1, '[^0-9]', '', 'g') <> ''
                    AND regexp_replace(phone_number, '[^0-9]', '', 'g')
                        LIKE '%' || regexp_replace($1, '[^0-9]', '', 'g') || '%'
                ))
           AND ($2::text IS NULL OR $2 = ANY(tags))
         ORDER BY display_name ASC
         LIMIT $3 OFFSET $4",
    )
    .bind(q.search.as_deref())
    .bind(q.tag.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("agent contacts list failed: {:#}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(rows))
}

async fn lookup_contact(
    State(state): State<AppState>,
    _perm: CanReadContacts,
    Path(number): Path<String>,
) -> Result<Json<Option<Contact>>, StatusCode> {
    let normalized = normalize_phone(&number);
    let row: Option<Contact> =
        sqlx::query_as("SELECT * FROM contacts WHERE phone_number = $1")
            .bind(&normalized)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(row))
}

async fn get_contact(
    State(state): State<AppState>,
    _perm: CanReadContacts,
    Path(id): Path<Uuid>,
) -> Result<Json<ContactWithCalls>, StatusCode> {
    let contact: Contact = sqlx::query_as("SELECT * FROM contacts WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let calls: Vec<crate::models::contact::ContactCallRow> = sqlx::query_as(
        "SELECT id, calldate, src, dst, duration, billsec, disposition,
                uniqueid, linkedid
         FROM cdr
         WHERE src = $1 OR dst = $1
         ORDER BY calldate DESC
         LIMIT 100",
    )
    .bind(&contact.phone_number)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let calls = crate::routes::contacts::enrich_calls_with_notes(&state, calls).await?;
    Ok(Json(ContactWithCalls { contact, calls }))
}

async fn create_contact(
    State(state): State<AppState>,
    _perm: CanReadContacts,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(input): Json<ContactInput>,
) -> Result<Json<Contact>, StatusCode> {
    let normalized = normalize_phone(&input.phone_number);
    if normalized.is_empty() || normalized.len() > 20 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.display_name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Some(email) = &input.email {
        if !email.is_empty() && !email.contains('@') {
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let created_by: Option<Uuid> = claims.sub.parse().ok();

    let row: Contact = sqlx::query_as(
        "INSERT INTO contacts
            (id, phone_number, display_name, email, company, notes, tags, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(&normalized)
    .bind(&input.display_name)
    .bind(&input.email)
    .bind(&input.company)
    .bind(&input.notes)
    .bind(&input.tags)
    .bind(created_by)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(db) = &e {
            if db.is_unique_violation() {
                return StatusCode::CONFLICT;
            }
        }
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(row))
}

/// Minimal list of active extensions (number + display name). Used by
/// the frontend directory so agents can resolve extension numbers to
/// names in their call history. No secrets, no config — just names.
#[derive(serde::Serialize)]
pub struct ExtensionLookup {
    pub number: String,
    pub display_name: String,
}

async fn list_all_extensions(
    State(state): State<AppState>,
    Extension(AuthUser(_claims)): Extension<AuthUser>,
) -> Result<Json<Vec<ExtensionLookup>>, StatusCode> {
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT number, display_name FROM extensions
         WHERE active = true ORDER BY number",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(
        rows.into_iter()
            .map(|(number, display_name)| ExtensionLookup {
                number,
                display_name,
            })
            .collect(),
    ))
}
