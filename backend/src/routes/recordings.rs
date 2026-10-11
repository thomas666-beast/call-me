use crate::auth::rbac::CanReadRecordings;
use crate::auth::signed_url;
use crate::auth::jwt::verify_access_token;
use crate::models::recording::Recording;
use crate::AppState;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use uuid::Uuid;

// -----------------------------------------------------------------------------
// Protected router — sits behind `require_auth` middleware.
// -----------------------------------------------------------------------------

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/recordings", get(list_recordings))
        .route("/recordings/:id", get(get_recording))
        .route("/recordings/:id/play-token", post(create_play_token))
        .route("/recordings/cleanup", post(trigger_cleanup))
}

// -----------------------------------------------------------------------------
// Public router — mounted WITHOUT `require_auth`. Does its own auth.
// -----------------------------------------------------------------------------

pub fn audio_router() -> Router<AppState> {
    Router::new().route("/recordings/:id/audio", get(stream_audio))
}

#[derive(Debug, Deserialize)]
struct RecordingQuery {
    from: Option<chrono::DateTime<chrono::Utc>>,
    to: Option<chrono::DateTime<chrono::Utc>>,
    src: Option<String>,
    dst: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    20
}

#[derive(Serialize)]
struct PlayTokenResponse {
    url: String,
    expires_at: i64,
}

#[derive(Deserialize, Default)]
struct AudioQuery {
    exp: Option<i64>,
    sig: Option<String>,
}

async fn list_recordings(
    State(state): State<AppState>,
    _perm: CanReadRecordings,
    Query(q): Query<RecordingQuery>,
) -> Result<Json<Vec<Recording>>, StatusCode> {
    let limit = q.limit.clamp(1, 500);
    let offset = q.offset.max(0);

    let rows: Vec<Recording> = sqlx::query_as(
        "SELECT * FROM recordings
         WHERE ($1::timestamptz IS NULL OR call_date >= $1)
           AND ($2::timestamptz IS NULL OR call_date <= $2)
           AND ($3::text IS NULL OR src = $3)
           AND ($4::text IS NULL OR dst = $4)
         ORDER BY indexed_at DESC
         LIMIT $5 OFFSET $6",
    )
    .bind(q.from)
    .bind(q.to)
    .bind(q.src.as_deref())
    .bind(q.dst.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("recordings list failed: {:#}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(rows))
}

async fn get_recording(
    State(state): State<AppState>,
    _perm: CanReadRecordings,
    Path(id): Path<Uuid>,
) -> Result<Json<Recording>, StatusCode> {
    let row: Recording = sqlx::query_as("SELECT * FROM recordings WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(row))
}

/// Create a short-lived signed URL for playback. Called by the frontend
/// before setting the `<audio>` src attribute.
async fn create_play_token(
    State(state): State<AppState>,
    _perm: CanReadRecordings,
    Path(id): Path<Uuid>,
) -> Result<Json<PlayTokenResponse>, StatusCode> {
    // Verify the recording exists so we don't sign URLs for nonexistent rows.
    let exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM recordings WHERE id = $1")
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if exists.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }

    let (expires_at, sig) = signed_url::sign(&state.cfg.jwt_secret, &id.to_string());
    let url = format!(
        "/api/admin/recordings/{}/audio?exp={}&sig={}",
        id, expires_at, sig
    );

    Ok(Json(PlayTokenResponse { url, expires_at }))
}

/// Stream the audio file with HTTP Range support.
///
/// Auth options (in order):
///   1. `Authorization: Bearer <jwt>` header (with `recordings.read`)
///   2. `?exp=<unix>&sig=<hmac>` query params (signed URL)
///
/// The signed URL path is what the browser's `<audio>` element uses,
/// since it can't send custom headers.
async fn stream_audio(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<AudioQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let recording_id = id.to_string();

    // --- Auth ---
    if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
        let header = auth_header.to_str().map_err(|_| StatusCode::UNAUTHORIZED)?;
        let token = header.strip_prefix("Bearer ").ok_or(StatusCode::UNAUTHORIZED)?;
        let claims = verify_access_token(&state.cfg.jwt_secret, token)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;
        let has: Option<(i32,)> = sqlx::query_as(
            "SELECT 1 FROM role_permissions rp
             JOIN permissions p ON p.id = rp.permission_id
             WHERE rp.role = $1 AND p.name = 'recordings.read'",
        )
        .bind(&claims.role)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if has.is_none() {
            return Err(StatusCode::FORBIDDEN);
        }
    } else if let (Some(exp), Some(sig)) = (q.exp, q.sig.as_deref()) {
        signed_url::verify(&state.cfg.jwt_secret, &recording_id, exp, sig)
            .map_err(|e| {
                tracing::warn!("Playback URL rejected: {}", e);
                StatusCode::UNAUTHORIZED
            })?;
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // --- Fetch the recording row ---
    let row: Recording = sqlx::query_as("SELECT * FROM recordings WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut file = File::open(&row.path).await.map_err(|e| {
        tracing::error!("failed to open recording {}: {:#}", row.path, e);
        StatusCode::NOT_FOUND
    })?;

    let metadata = file
        .metadata()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let file_size = metadata.len();

    let content_type = match row.format.as_str() {
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "gsm" => "audio/gsm",
        _ => "application/octet-stream",
    };

    let range_header = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    if let Some(range) = range_header {
        let (start, end) = match parse_range(&range, file_size) {
            Some(v) => v,
            None => {
                let mut resp = StatusCode::RANGE_NOT_SATISFIABLE.into_response();
                resp.headers_mut().insert(
                    header::CONTENT_RANGE,
                    HeaderValue::from_str(&format!("bytes */{}", file_size)).unwrap(),
                );
                return Ok(resp);
            }
        };

        let length = end - start + 1;
        file.seek(SeekFrom::Start(start))
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut buffer = vec![0u8; length as usize];
        file.read_exact(&mut buffer)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let mut resp = Response::new(Body::from(buffer));
        *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
        resp.headers_mut()
            .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
        resp.headers_mut().insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(&format!("bytes {}-{}/{}", start, end, file_size)).unwrap(),
        );
        resp.headers_mut().insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&length.to_string()).unwrap(),
        );
        resp.headers_mut()
            .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
        return Ok(resp);
    }

    let mut buffer = Vec::with_capacity(file_size as usize);
    file.read_to_end(&mut buffer)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut resp = Response::new(Body::from(buffer));
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    resp.headers_mut().insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&file_size.to_string()).unwrap(),
    );
    resp.headers_mut()
        .insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    Ok(resp)
}

/// Manually trigger the retention task. Useful for testing and for admins
/// who want to free disk immediately after changing the retention policy.
async fn trigger_cleanup(
    State(state): State<AppState>,
    _perm: CanReadRecordings,
) -> Result<Json<crate::recordings_retention::RetentionReport>, StatusCode> {
    match crate::recordings_retention::run_once(&state).await {
        Ok(report) => Ok(Json(report)),
        Err(e) => {
            tracing::error!("Manual recordings cleanup failed: {:#}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

fn parse_range(value: &str, file_size: u64) -> Option<(u64, u64)> {
    let value = value.strip_prefix("bytes=")?;
    let mut parts = value.split('-');
    let start_str = parts.next()?;
    let end_str = parts.next()?;

    let start: u64 = if start_str.is_empty() {
        let suffix: u64 = end_str.parse().ok()?;
        file_size.saturating_sub(suffix)
    } else {
        start_str.parse().ok()?
    };

    let end: u64 = if end_str.is_empty() {
        file_size.saturating_sub(1)
    } else {
        end_str.parse().ok()?
    };

    if start > end || start >= file_size {
        return None;
    }
    Some((start, end.min(file_size - 1)))
}
