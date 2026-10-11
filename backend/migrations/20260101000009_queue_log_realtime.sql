-- ============================================================================
-- queue_log — Asterisk Realtime queue event log
-- ============================================================================
--
-- Asterisk writes to this table directly via res_config_pgsql, mapped in
-- extconfig.conf with:  queue_log => pgsql,callcenter
--
-- Column names MUST match what Asterisk's ast_store_realtime() call sends:
--   time, callid, queuename, agent, event, data1, data2, data3
--
-- The old queue_log table (from migration 08) is dropped — it was for
-- a file-tailing approach we're not using.

DROP TABLE IF EXISTS queue_log CASCADE;

CREATE TABLE queue_log (
    id         BIGSERIAL PRIMARY KEY,
    "time"     BIGINT NOT NULL,
    callid     TEXT NOT NULL,
    queuename  TEXT NOT NULL,
    agent      TEXT NOT NULL,
    event      TEXT NOT NULL,
    data1      TEXT NOT NULL DEFAULT '',
    data2      TEXT NOT NULL DEFAULT '',
    data3      TEXT NOT NULL DEFAULT ''
);

CREATE INDEX idx_queue_log_time      ON queue_log ("time" DESC);
CREATE INDEX idx_queue_log_callid    ON queue_log (callid);
CREATE INDEX idx_queue_log_queuename ON queue_log (queuename);
CREATE INDEX idx_queue_log_agent     ON queue_log (agent);
CREATE INDEX idx_queue_log_event     ON queue_log (event);
