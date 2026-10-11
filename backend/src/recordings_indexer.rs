//! Background task that indexes recording files.
//!
//! Every 60 seconds, scans the recording directory. For each new `.wav`
//! file it finds, it inserts a row into the `recordings` table, matching
//! the file's `uniqueid` (the filename stem) to a CDR row.
//!
//! Asterisk writes recordings named `<uniqueid>.wav` inside the container,
//! which appears at `asterisk-recordings/<uniqueid>.wav` on the host thanks
//! to the bind mount.

use crate::AppState;
use anyhow::Result;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use uuid::Uuid;

/// Where recordings live on the host. This path is visible to the backend
/// because it runs on the host; the container sees the same files at
/// /var/spool/asterisk/monitor via the bind mount.
fn recordings_dir() -> PathBuf {
    PathBuf::from("../asterisk-recordings")
}

pub async fn run(state: AppState) {
    tracing::info!("Recordings indexer started");

    let mut ticker = tokio::time::interval(Duration::from_secs(60));
    loop {
        ticker.tick().await;
        if let Err(e) = scan_once(&state).await {
            tracing::error!("Recordings indexer scan failed: {:#}", e);
        }
    }
}

async fn scan_once(state: &AppState) -> Result<()> {
    let dir = recordings_dir();
    if !dir.exists() {
        return Ok(());
    }

    let mut entries = fs::read_dir(&dir).await?;
    let mut indexed = 0usize;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext != "wav" && ext != "mp3" && ext != "ogg" && ext != "gsm" {
            continue;
        }

        let filename = match path.file_name().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        // Filename stem is the Asterisk UNIQUEID of the calling channel.
        let uniqueid = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };

        // Skip if already indexed.
        let exists: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM recordings WHERE filename = $1")
                .bind(&filename)
                .fetch_optional(&state.pool)
                .await?;
        if exists.is_some() {
            continue;
        }

        let metadata = fs::metadata(&path).await?;
        let size_bytes = metadata.len() as i64;
        let mtime = metadata.modified().ok();

        // Try to find the matching CDR. Match against uniqueid OR linkedid,
        // because Asterisk assigns different uniqueids to the two channels
        // of a bridged call, and MixMonitor uses the calling channel's id.
        let cdr: Option<(String, String, String, String, chrono::DateTime<chrono::Utc>)> =
            sqlx::query_as(
                "SELECT uniqueid, linkedid, src, dst, calldate
                 FROM cdr
                 WHERE uniqueid = $1 OR linkedid = $1
                 ORDER BY calldate DESC
                 LIMIT 1",
            )
            .bind(&uniqueid)
            .fetch_optional(&state.pool)
            .await?;

        let (uniqueid_val, linkedid_val, src, dst, call_date) = match cdr {
            Some((u, l, s, d, c)) => (u, l, s, d, Some(c)),
            None => (uniqueid.clone(), String::new(), String::new(), String::new(), None),
        };

        let path_str = path.to_string_lossy().to_string();

        let format = ext;
        let file_mtime: Option<chrono::DateTime<chrono::Utc>> = mtime
            .map(chrono::DateTime::<chrono::Utc>::from);

        sqlx::query(
            "INSERT INTO recordings
                (id, uniqueid, linkedid, path, filename, format, size_bytes,
                 call_date, src, dst, file_mtime)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(Uuid::new_v4())
        .bind(&uniqueid_val)
        .bind(&linkedid_val)
        .bind(&path_str)
        .bind(&filename)
        .bind(&format)
        .bind(size_bytes)
        .bind(call_date)
        .bind(&src)
        .bind(&dst)
        .bind(file_mtime)
        .execute(&state.pool)
        .await?;

        indexed += 1;
        tracing::info!("Indexed recording: {} ({} bytes)", filename, size_bytes);
    }

    if indexed > 0 {
        tracing::info!("Recordings indexer: {} new file(s) indexed", indexed);
    }
    Ok(())
}
