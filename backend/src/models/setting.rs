use anyhow::Result;
use sqlx::PgPool;

/// Read a setting. Returns None if not present.
pub async fn get(pool: &PgPool, key: &str) -> Result<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = $1")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(v,)| v))
}

/// Read an integer setting. Returns `default` if missing or unparseable.
pub async fn get_int(pool: &PgPool, key: &str, default: i64) -> Result<i64> {
    match get(pool, key).await? {
        Some(v) => Ok(v.parse().unwrap_or(default)),
        None => Ok(default),
    }
}

/// Write a setting (insert or update).
pub async fn set(pool: &PgPool, key: &str, value: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO settings (key, value, updated_at)
         VALUES ($1, $2, now())
         ON CONFLICT (key) DO UPDATE
           SET value = EXCLUDED.value, updated_at = now()",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub const KEY_RECORDINGS_RETENTION_DAYS: &str = "recordings.retention_days";
