use crate::auth::rbac::CanReadCdr;
use crate::models::cdr::{CallSummary, Cdr, CdrQuery, CdrWithCel, QueueEvent};
use crate::models::cel::Cel;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cdr", get(list_cdr))
        .route("/cdr/:id", get(get_cdr))
}

async fn list_cdr(
    State(state): State<AppState>,
    _perm: CanReadCdr,
    Query(q): Query<CdrQuery>,
) -> Result<Json<Vec<Cdr>>, StatusCode> {
    let limit = q.limit.clamp(1, 500);
    let offset = q.offset.max(0);

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
                END AS kind,

                CASE
                    WHEN EXISTS (
                        SELECT 1 FROM queue_log
                        WHERE (callid = c.uniqueid OR callid = c.linkedid)
                          AND event = 'CONNECT'
                    ) THEN 'answered:' || COALESCE((
                        SELECT split_part(agent, '/', 2)
                        FROM queue_log
                        WHERE (callid = c.uniqueid OR callid = c.linkedid)
                          AND event = 'CONNECT'
                        ORDER BY \"time\" ASC
                        LIMIT 1
                    ), '')
                    WHEN EXISTS (
                        SELECT 1 FROM queue_log
                        WHERE (callid = c.uniqueid OR callid = c.linkedid)
                          AND event = 'ABANDON'
                    ) THEN 'abandoned'
                    WHEN EXISTS (
                        SELECT 1 FROM queue_log
                        WHERE (callid = c.uniqueid OR callid = c.linkedid)
                          AND event = 'RINGCANCELED'
                    ) THEN 'cancelled'
                    WHEN EXISTS (
                        SELECT 1 FROM queue_log
                        WHERE (callid = c.uniqueid OR callid = c.linkedid)
                          AND event = 'EXITWITHTIMEOUT'
                    ) THEN 'timed_out'
                    WHEN EXISTS (
                        SELECT 1 FROM queue_log
                        WHERE (callid = c.uniqueid OR callid = c.linkedid)
                          AND event = 'RINGNOANSWER'
                    ) THEN 'missed'
                    ELSE NULL
                END AS outcome,

                CASE
                    WHEN EXISTS (
                        SELECT 1 FROM queue_log
                        WHERE (callid = c.uniqueid OR callid = c.linkedid)
                          AND event = 'CONNECT'
                    ) THEN COALESCE((
                        SELECT split_part(agent, '/', 2)
                        FROM queue_log
                        WHERE (callid = c.uniqueid OR callid = c.linkedid)
                          AND event = 'CONNECT'
                        ORDER BY \"time\" ASC
                        LIMIT 1
                    ), '')
                    ELSE NULL
                END AS answered_by,

                CASE
                    -- Queue call: the agent who answered
                    WHEN EXISTS (SELECT 1 FROM queues q WHERE q.number = c.dst) THEN (
                        SELECT split_part(agent, '/', 2)
                        FROM queue_log
                        WHERE (callid = c.uniqueid OR callid = c.linkedid)
                          AND event = 'CONNECT'
                        ORDER BY \"time\" ASC
                        LIMIT 1
                    )
                    -- Outbound call: the extension who dialed
                    WHEN EXISTS (SELECT 1 FROM extensions e WHERE e.number = c.src)
                     AND NOT EXISTS (SELECT 1 FROM extensions e WHERE e.number = c.dst)
                        THEN c.src
                    -- Everything else (internal, external): no single agent
                    ELSE NULL
                END AS agent_extension
            FROM cdr c
            WHERE ($1::timestamptz IS NULL OR c.calldate >= $1)
              AND ($2::timestamptz IS NULL OR c.calldate <= $2)
              AND ($3::text       IS NULL OR c.src = $3)
              AND ($4::text       IS NULL OR c.dst = $4)
              AND ($5::text       IS NULL OR c.disposition = $5)
            ORDER BY
                COALESCE(NULLIF(c.linkedid, ''), c.uniqueid),
                CASE WHEN EXISTS (
                    SELECT 1 FROM extensions e WHERE e.number = c.src
                ) THEN 1 ELSE 0 END ASC,
                c.calldate DESC,
                c.id ASC
        ) sub
        WHERE ($6::boolean = true OR kind <> 'internal')
        ORDER BY calldate DESC, id DESC
        LIMIT $7 OFFSET $8",
    )
    .bind(q.from)
    .bind(q.to)
    .bind(q.src.as_deref())
    .bind(q.dst.as_deref())
    .bind(q.disposition.as_deref())
    .bind(q.include_internal)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("cdr list query failed: {:#}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Normalize empty strings to None for cleaner JSON.
    let rows = rows
        .into_iter()
        .map(|mut c| {
            if let Some(ref ab) = c.answered_by {
                if ab.is_empty() {
                    c.answered_by = None;
                }
            }
            if let Some(ref ae) = c.agent_extension {
                if ae.is_empty() {
                    c.agent_extension = None;
                }
            }
            c
        })
        .collect();

    Ok(Json(rows))
}

#[derive(FromRow)]
struct QueueLogRow {
    event: String,
    time: DateTime<Utc>,
    queuename: String,
    agent: String,
    data: String,
}

async fn get_cdr(
    State(state): State<AppState>,
    _perm: CanReadCdr,
    Path(id): Path<i64>,
) -> Result<Json<CdrWithCel>, StatusCode> {
    let cdr: Cdr = sqlx::query_as("SELECT * FROM cdr WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let cel: Vec<Cel> = sqlx::query_as(
        "SELECT * FROM cel
         WHERE linkedid = $1
         ORDER BY eventtime ASC, id ASC",
    )
    .bind(&cdr.linkedid)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ql_rows: Vec<QueueLogRow> = sqlx::query_as(
        "SELECT event, \"time\", queuename, agent, data
         FROM queue_log
         WHERE callid = $1 OR callid = $2
         ORDER BY \"time\" ASC, id ASC",
    )
    .bind(&cdr.uniqueid)
    .bind(&cdr.linkedid)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let queue_events: Vec<QueueEvent> = ql_rows
        .into_iter()
        .map(|r| {
            let agent_extension = if r.agent == "NONE" || r.agent.is_empty() {
                None
            } else {
                r.agent.split('/').next_back().map(|s| s.to_string())
            };
            QueueEvent {
                event: r.event,
                time: r.time,
                queue_number: r.queuename,
                agent_extension,
                data: r.data,
            }
        })
        .collect();

    let summary = classify_call(&state, &cdr, &queue_events)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CdrWithCel {
        cdr,
        cel,
        queue_events,
        summary,
    }))
}

async fn classify_call(
    state: &AppState,
    cdr: &Cdr,
    queue_events: &[QueueEvent],
) -> Result<CallSummary, sqlx::Error> {
    let is_queue: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM queues WHERE number = $1")
            .bind(&cdr.dst)
            .fetch_optional(&state.pool)
            .await?;

    let is_src_ext: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM extensions WHERE number = $1")
            .bind(&cdr.src)
            .fetch_optional(&state.pool)
            .await?;

    let is_dst_ext: Option<(i32,)> =
        sqlx::query_as("SELECT 1 FROM extensions WHERE number = $1")
            .bind(&cdr.dst)
            .fetch_optional(&state.pool)
            .await?;

    let kind = if is_queue.is_some() {
        "queue"
    } else if is_src_ext.is_some() && is_dst_ext.is_some() {
        "internal"
    } else if is_src_ext.is_some() {
        "outbound"
    } else {
        "external"
    };

    let mut queue_number: Option<String> = None;
    let (outcome, answered_by) = if kind == "queue" {
        queue_number = Some(cdr.dst.clone());

        if let Some(c) = queue_events.iter().find(|e| e.event == "CONNECT") {
            let ext = c.agent_extension.clone();
            let outcome = ext
                .as_ref()
                .map(|e| format!("answered:{}", e))
                .unwrap_or_else(|| "answered".to_string());
            (Some(outcome), ext)
        } else if queue_events.iter().any(|e| e.event == "ABANDON") {
            (Some("abandoned".to_string()), None)
        } else if queue_events.iter().any(|e| e.event == "RINGCANCELED") {
            (Some("cancelled".to_string()), None)
        } else if queue_events.iter().any(|e| e.event == "EXITWITHTIMEOUT") {
            (Some("timed_out".to_string()), None)
        } else if queue_events.iter().any(|e| e.event == "RINGNOANSWER") {
            (Some("missed".to_string()), None)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Ok(CallSummary {
        kind: kind.to_string(),
        outcome,
        answered_by,
        queue_number,
    })
}
