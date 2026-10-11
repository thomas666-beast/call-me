use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Cdr {
    pub id: i64,
    pub calldate: DateTime<Utc>,
    pub start_time: Option<DateTime<Utc>>,
    pub answer_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,

    pub clid: String,
    pub src: String,
    pub dst: String,
    pub dcontext: String,
    pub channel: String,
    pub dstchannel: String,
    pub lastapp: String,
    pub lastdata: String,

    pub duration: i32,
    pub billsec: i32,

    pub disposition: String,
    pub amaflags: i32,
    pub accountcode: String,

    pub uniqueid: String,
    pub userfield: String,

    pub peeraccount: String,
    pub linkedid: String,
    pub sequence: i32,

    #[sqlx(default)]
    #[serde(default)]
    pub kind: Option<String>,

    #[sqlx(default)]
    #[serde(default)]
    pub outcome: Option<String>,

    #[sqlx(default)]
    #[serde(default)]
    pub answered_by: Option<String>,

    /// The agent extension involved in this call, if there is one.
    /// - Queue call → the agent who answered
    /// - Outbound call → the agent who dialed
    /// - Internal call → null (both parties are agents, already shown
    ///   in the Caller and Destination columns)
    /// - External call → null
    #[sqlx(default)]
    #[serde(default)]
    pub agent_extension: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CdrQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub src: Option<String>,
    pub dst: Option<String>,
    pub disposition: Option<String>,
    #[serde(default = "default_true")]
    pub include_internal: bool,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_true() -> bool {
    true
}

fn default_limit() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct QueueEvent {
    pub event: String,
    pub time: DateTime<Utc>,
    pub queue_number: String,
    pub agent_extension: Option<String>,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct CallSummary {
    pub kind: String,
    pub outcome: Option<String>,
    pub answered_by: Option<String>,
    pub queue_number: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CdrWithCel {
    pub cdr: Cdr,
    pub cel: Vec<crate::models::cel::Cel>,
    pub queue_events: Vec<QueueEvent>,
    pub summary: CallSummary,
}
