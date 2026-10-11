use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A single CEL (Channel Event Logging) row. Written by Asterisk's
/// cel_pgsql.so, read by us. Many rows per call.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Cel {
    pub id: i64,
    pub eventtype: String,
    pub eventtime: DateTime<Utc>,

    pub userdeftype: String,
    pub cid_name: String,
    pub cid_num: String,
    pub cid_ani: String,
    pub cid_rdnis: String,
    pub cid_dnid: String,

    pub exten: String,
    pub context: String,
    pub channame: String,
    pub appname: String,
    pub appdata: String,

    pub amaflags: i32,
    pub accountcode: String,
    pub peer: String,
    pub userfield: String,

    pub uniqueid: String,
    pub linkedid: String,

    pub extra: String,
}
