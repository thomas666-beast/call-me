use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub extension: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PublicUser {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub role: String,
    pub extension: Option<String>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for PublicUser {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            email: u.email,
            role: u.role,
            extension: u.extension,
            active: u.active,
            created_at: u.created_at,
        }
    }
}

/// What the API accepts when creating a panel user (admin or supervisor).
#[derive(Debug, Deserialize)]
pub struct UserInput {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub role: String,
    #[serde(default = "default_true")]
    pub active: bool,
}

/// What the API accepts when updating a panel user.
#[derive(Debug, Deserialize)]
pub struct UserUpdate {
    pub email: Option<String>,
    pub password: Option<String>,
    pub active: Option<bool>,
}

fn default_true() -> bool {
    true
}
