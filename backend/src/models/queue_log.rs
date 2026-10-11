//! Queue log model.
//!
//! The queue_log table is written to by Asterisk (via res_config_pgsql)
//! and read by the agent panel through raw SQL in routes/agent.rs.
//!
//! No typed struct here yet — when we add a queue-log list endpoint,
//! we'll add a `QueueLogEntry` struct with `FromRow`. Until then,
//! keeping an unused struct only produces dead_code warnings.
