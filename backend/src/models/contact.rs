use crate::models::call_note::PublicCallNote;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
    pub phone_number: String,
    pub display_name: String,
    pub email: Option<String>,
    pub company: Option<String>,
    pub notes: String,
    pub tags: Vec<String>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ContactInput {
    pub phone_number: String,
    pub display_name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub company: Option<String>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ContactQuery {
    pub search: Option<String>,
    pub tag: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

/// DB row for one call in a contact's history (before enrichment).
#[derive(Debug, Clone, FromRow)]
pub struct ContactCallRow {
    pub id: i64,
    pub calldate: DateTime<Utc>,
    pub src: String,
    pub dst: String,
    pub duration: i32,
    pub billsec: i32,
    pub disposition: String,
    pub uniqueid: String,
    pub linkedid: String,
}

/// One call with its attached notes. This is what the API returns.
#[derive(Debug, Serialize)]
pub struct ContactCallDetail {
    pub id: i64,
    pub calldate: DateTime<Utc>,
    pub src: String,
    pub dst: String,
    pub duration: i32,
    pub billsec: i32,
    pub disposition: String,
    pub uniqueid: String,
    pub linkedid: String,
    pub notes: Vec<PublicCallNote>,
}

#[derive(Debug, Serialize)]
pub struct ContactWithCalls {
    pub contact: Contact,
    pub calls: Vec<ContactCallDetail>,
}

/// Normalize a phone number for storage and matching.
pub fn normalize_phone(input: &str) -> String {
    let trimmed = input.trim();
    let has_plus = trimmed.starts_with('+');
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    if has_plus {
        format!("+{}", digits)
    } else {
        digits
    }
}
