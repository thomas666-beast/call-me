use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// What the API returns for one agent.
#[derive(Debug, Serialize)]
pub struct AgentView {
    /// The user's id. Also used as the `:id` in all agent endpoints.
    pub id: Uuid,
    pub username: String,
    pub extension_number: Option<String>,
    pub display_name: Option<String>,
    pub sip_secret: Option<String>,
    pub queue_numbers: Vec<String>,
    pub active: bool,
}

/// What the API accepts when creating an agent.
///
/// The username is equal to the extension number — the admin does not
/// enter them separately.
#[derive(Debug, Deserialize)]
pub struct AgentInput {
    pub extension_number: String,
    pub display_name: String,
    pub sip_secret: String,
    pub password: String,
    #[serde(default)]
    pub queue_numbers: Vec<String>,
    #[serde(default = "default_true")]
    pub active: bool,
}

/// What the API accepts when updating an agent. All fields optional.
#[derive(Debug, Deserialize)]
pub struct AgentUpdate {
    pub display_name: Option<String>,
    pub sip_secret: Option<String>,
    pub password: Option<String>,
    pub queue_numbers: Option<Vec<String>>,
    pub active: Option<bool>,
}

fn default_true() -> bool {
    true
}
