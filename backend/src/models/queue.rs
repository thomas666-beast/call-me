use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A queue row.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Queue {
    pub id: Uuid,
    pub number: String,
    pub name: String,
    pub strategy: String,
    pub timeout: i32,
    pub retry: i32,
    pub maxlen: i32,
    pub acw_seconds: i32,
    pub musicclass: String,
    pub joinempty: bool,
    pub leavewhenempty: bool,
    pub ringinuse: bool,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// What the API returns. Includes the list of member extension numbers.
#[derive(Debug, Serialize)]
pub struct PublicQueue {
    pub id: Uuid,
    pub number: String,
    pub name: String,
    pub strategy: String,
    pub timeout: i32,
    pub retry: i32,
    pub maxlen: i32,
    pub acw_seconds: i32,
    pub musicclass: String,
    pub joinempty: bool,
    pub leavewhenempty: bool,
    pub ringinuse: bool,
    pub active: bool,
    pub members: Vec<String>,
}

/// What the API accepts.
#[derive(Debug, Deserialize)]
pub struct QueueInput {
    pub number: String,
    pub name: String,
    #[serde(default = "default_strategy")]
    pub strategy: String,
    #[serde(default = "default_timeout")]
    pub timeout: i32,
    #[serde(default = "default_retry")]
    pub retry: i32,
    #[serde(default = "default_maxlen")]
    pub maxlen: i32,
    #[serde(default = "default_acw")]
    pub acw_seconds: i32,
    #[serde(default = "default_musicclass")]
    pub musicclass: String,
    #[serde(default = "default_true")]
    pub joinempty: bool,
    #[serde(default)]
    pub leavewhenempty: bool,
    #[serde(default)]
    pub ringinuse: bool,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(default)]
    pub members: Vec<String>,
}

fn default_strategy() -> String {
    "ringall".to_string()
}
fn default_timeout() -> i32 {
    15
}
fn default_retry() -> i32 {
    5
}
fn default_maxlen() -> i32 {
    10
}
fn default_acw() -> i32 {
    15
}
fn default_musicclass() -> String {
    "default".to_string()
}
fn default_true() -> bool {
    true
}
