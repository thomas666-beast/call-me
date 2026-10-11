//! Background task that deletes recordings older than the retention policy.
//!
//! Runs once 30 seconds after startup, then every 24 hours.
//! Admins can also trigger it immediately via
//! `POST /api/admin/recordings/cleanup`.
//!
//! Deletion order per recording:
//!   1. Try to delete the file on disk. If it's already missing, proceed.
//!   2. Delete the DB row.
//!
//! If the file delete fails for any reason other than "not found",
//! we log and skip the DB row so it will be retried on the next run.

use crate::models::setting;
use crate::AppState;
use anyhow::Result;
use chrono::{Duration, Utc};
use std::time::Duration as StdDuration;
use tokio::fs;
use uuid::Uuid;

pub async fn run(state: AppState) {
    // Wait for the rest of the system to be ready.
    tokio::time::sleep(StdDuration::from_secs(30)).await;

    loop {
        if let Err(e) = run_once(&state).await {
            tracing::error!("Recordings retention task failed: {:#}", e);
        }
        tokio::time::sleep(StdDuration::from_secs(24 * 3600)).await;
    }
}

/// Public entry point for the manual-cleanup endpoint.
pub async fn run_once(state: &AppState) -> Result<RetentionReport> {
    let days = setting::get_int(
        &state.pool,
        setting::KEY_RECORDINGS_RETENTION_DAYS,
        0,
    )
    .await?;

    if days <= 0 {
        tracing::debug!("Recordings retention disabled (keep forever)");
        return Ok(RetentionReport {
            deleted: 0,
            skipped: 0,
            retention_days: 0,
        });
    }

    let cutoff = Utc::now() - Duration::days(days);

    // Select recordings whose effective call time is older than the cutoff.
    // We fall back to indexed_at if call_date is NULL.
    let rows: Vec<(Uuid, String, String)> = sqlx::query_as(
        "SELECT id, path, filename FROM recordings
         WHERE COALESCE(call_date, indexed_at) < $1",
    )
    .bind(cutoff)
    .fetch_all(&state.pool)
    .await?;

    let mut deleted = 0usize;
    let mut skipped = 0usize;

    for (id, path, filename) in rows {
        // Try to delete the file. Missing is OK, other errors are not.
        match fs::remove_file(&path).await {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!("Recording file already gone: {}", path);
            }
            Err(e) => {
                tracing::warn!(
                    "Could not delete recording file {} ({}): {:#}",
                    filename,
                    path,
                    e
                );
                skipped += 1;
                continue;
            }
        }

        // File is gone (or was already gone). Remove the DB row.
        if let Err(e) = sqlx::query("DELETE FROM recordings WHERE id = $1")
            .bind(id)
            .execute(&state.pool)
            .await
        {
            tracing::error!("Failed to delete recording row {}: {:#}", id, e);
            skipped += 1;
            continue;
        }

        deleted += 1;
    }

    if deleted > 0 || skipped > 0 {
        tracing::info!(
            "Recordings retention: {} deleted, {} skipped (retention {} days)",
            deleted,
            skipped,
            days
        );
    }

    Ok(RetentionReport {
        deleted,
        skipped,
        retention_days: days,
    })
}

#[derive(Debug, serde::Serialize)]
pub struct RetentionReport {
    pub deleted: usize,
    pub skipped: usize,
    pub retention_days: i64,
}
