-- ============================================================================
-- queue_log — corrected schema for Asterisk Realtime writes
-- ============================================================================
--
-- Asterisk 22's res_config_pgsql writes:
--
--   INSERT INTO queue_log (time, data, event, agent, queuename, callid)
--   VALUES ('2026-09-24 18:40:00.870416', '14|1790275186.3|14',
--           'CONNECT', 'PJSIP/1002', '2000', '1790275186.2')
--
-- So the columns are: time (timestamptz), data (pipe-delimited text),
-- event, agent, queuename, callid. Migration 09 had the wrong shape.
-- This migration replaces it.

DROP TABLE IF EXISTS queue_log CASCADE;

CREATE TABLE queue_log (
    id         BIGSERIAL PRIMARY KEY,
    "time"     TIMESTAMPTZ NOT NULL,
    data       TEXT NOT NULL DEFAULT '',
    event      TEXT NOT NULL,
    agent      TEXT NOT NULL,
    queuename  TEXT NOT NULL,
    callid     TEXT NOT NULL
);

CREATE INDEX idx_queue_log_time      ON queue_log ("time" DESC);
CREATE INDEX idx_queue_log_callid    ON queue_log (callid);
CREATE INDEX idx_queue_log_queuename ON queue_log (queuename);
CREATE INDEX idx_queue_log_agent     ON queue_log (agent);
CREATE INDEX idx_queue_log_event     ON queue_log (event);
