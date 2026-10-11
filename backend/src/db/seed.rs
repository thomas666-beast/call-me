use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::password;

pub async fn dev_users(pool: &PgPool) -> Result<()> {
    let users = [
        ("admin", "admin", "admin", None::<&str>),
        ("agent1", "agent1", "agent", Some("1001")),
    ];

    for (username, password_plain, role, extension) in users {
        let exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(pool)
            .await?;

        if exists.is_some() {
            tracing::info!("Dev user '{}' already exists, skipping", username);
            continue;
        }

        let hash = password::hash_password(password_plain)?;

        sqlx::query(
            "INSERT INTO users (id, username, password_hash, role, extension)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(Uuid::new_v4())
        .bind(username)
        .bind(&hash)
        .bind(role)
        .bind(extension)
        .execute(pool)
        .await?;

        tracing::info!("Seeded dev user '{}' (role: {})", username, role);
    }

    Ok(())
}
