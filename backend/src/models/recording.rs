use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A call recording file. Written by OUR backend, not by Asterisk.
///
/// One row per audio file on disk. The `path` column is the absolute path
/// on the host, which the backend serves directly (the container sees the
/// same path through the bind mount).
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Recording {
    pub id: Uuid,
    pub uniqueid: String,
    pub linkedid: String,

    pub path: String,
    pub filename: String,
    pub format: String,
    pub size_bytes: i64,
    pub duration_secs: Option<i32>,

    pub call_date: Option<DateTime<Utc>>,
    pub src: String,
    pub dst: String,
    pub queue_number: Option<String>,
    pub agent_extension: Option<String>,

    pub indexed_at: DateTime<Utc>,
    pub file_mtime: Option<DateTime<Utc>>,

    pub last_played_at: Option<DateTime<Utc>>,
    pub last_played_by: Option<Uuid>,
}
