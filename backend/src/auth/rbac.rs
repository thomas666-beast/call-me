use crate::auth::middleware::AuthUser;
use crate::AppState;
use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

async fn check_permission(
    state: &AppState,
    parts: &mut Parts,
    permission: &str,
) -> Result<(), StatusCode> {
    let user = parts
        .extensions
        .get::<AuthUser>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let has: Option<(i32,)> = sqlx::query_as(
        "SELECT 1 FROM role_permissions rp
         JOIN permissions p ON p.id = rp.permission_id
         WHERE rp.role = $1 AND p.name = $2",
    )
    .bind(&user.0.role)
    .bind(permission)
    .fetch_optional(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if has.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(())
}

macro_rules! permission_struct {
    ($name:ident, $perm:literal) => {
        pub struct $name;
        #[async_trait]
        impl FromRequestParts<AppState> for $name {
            type Rejection = StatusCode;
            async fn from_request_parts(
                parts: &mut Parts,
                state: &AppState,
            ) -> Result<Self, Self::Rejection> {
                check_permission(state, parts, $perm).await?;
                Ok($name)
            }
        }
    };
}

permission_struct!(CanReadUsers, "users.read");
permission_struct!(CanWriteUsers, "users.write");
permission_struct!(CanReadConfig, "config.read");
permission_struct!(CanWriteConfig, "config.write");
permission_struct!(CanReadQueues, "queues.read");
permission_struct!(CanWriteQueues, "queues.write");
permission_struct!(CanReadCdr, "cdr.read");
permission_struct!(CanReadRecordings, "recordings.read");
permission_struct!(CanReadContacts, "contacts.read");
permission_struct!(CanWriteContacts, "contacts.write");
permission_struct!(CanReadAgents, "agents.read");
permission_struct!(CanWriteAgents, "agents.write");
