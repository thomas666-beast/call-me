use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A SIP extension managed by the admin UI.
///
/// This struct maps 1:1 to the `extensions` table. `FromRow` is what lets
/// `sqlx::query_as::<_, Extension>(...)` build one of these from a row.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Extension {
    pub id: Uuid,
    pub number: String,
    pub display_name: String,
    pub secret: String,
    pub context: String,
    pub transport: String,
    pub codecs: Vec<String>,
    pub max_contacts: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// What the API returns when listing or fetching an extension.
///
/// Note: `secret` is included here because the admin UI needs to be able to
/// view it when editing an extension. In a more restrictive system you'd
/// omit it and only allow setting, not reading. For Phase 3a, admins see it.
#[derive(Debug, Serialize)]
pub struct PublicExtension {
    pub id: Uuid,
    pub number: String,
    pub display_name: String,
    pub secret: String,
    pub context: String,
    pub transport: String,
    pub codecs: Vec<String>,
    pub max_contacts: i32,
    pub active: bool,
}

impl From<Extension> for PublicExtension {
    fn from(e: Extension) -> Self {
        Self {
            id: e.id,
            number: e.number,
            display_name: e.display_name,
            secret: e.secret,
            context: e.context,
            transport: e.transport,
            codecs: e.codecs,
            max_contacts: e.max_contacts,
            active: e.active,
        }
    }
}

/// What the API accepts when creating or updating an extension.
///
/// `number` is validated server-side. `codecs` is a list; the DB stores it
/// as TEXT[]. `max_contacts` defaults to 1 if omitted.
#[derive(Debug, Deserialize)]
pub struct ExtensionInput {
    pub number: String,
    pub display_name: String,
    pub secret: String,
    #[serde(default = "default_context")]
    pub context: String,
    #[serde(default = "default_transport")]
    pub transport: String,
    #[serde(default = "default_codecs")]
    pub codecs: Vec<String>,
    #[serde(default = "default_max_contacts")]
    pub max_contacts: i32,
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_context() -> String {
    "from-internal".to_string()
}

fn default_transport() -> String {
    "transport-udp".to_string()
}

fn default_codecs() -> Vec<String> {
    vec!["ulaw".to_string(), "alaw".to_string()]
}

fn default_max_contacts() -> i32 {
    1
}

fn default_active() -> bool {
    true
}
