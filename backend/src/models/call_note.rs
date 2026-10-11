use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CallNoteInput {
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PublicCallNote {
    pub id: Uuid,
    pub call_uniqueid: String,
    pub call_linkedid: String,
    pub author_id: Uuid,
    pub author_username: String,
    pub note: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Joined row used by both `routes/agent.rs` and `routes/contacts.rs`.
#[derive(Debug, Clone, FromRow)]
pub struct NoteRow {
    pub id: Uuid,
    pub call_uniqueid: String,
    pub call_linkedid: String,
    pub author_id: Uuid,
    pub author_username: String,
    pub note: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl NoteRow {
    pub fn into_public(self) -> PublicCallNote {
        PublicCallNote {
            id: self.id,
            call_uniqueid: self.call_uniqueid,
            call_linkedid: self.call_linkedid,
            author_id: self.author_id,
            author_username: self.author_username,
            note: self.note,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
