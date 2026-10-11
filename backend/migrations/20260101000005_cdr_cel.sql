-- ============================================================================
-- CDR and CEL tables
-- ============================================================================
--
-- These tables are written to DIRECTLY by Asterisk's cdr_pgsql.so and
-- cel_pgsql.so modules. The backend only reads from them.
--
-- Column names and types must match what Asterisk expects, exactly.
-- Asterisk inserts rows by column name, so if a column is missing, the
-- insert fails silently (Asterisk logs an error, but the call still
-- completes normally).
--
-- Schema verified against:
--   Asterisk 22 cdr_pgsql / cel_pgsql column lists
--   https://docs.asterisk.org/Latest_API/.../CDR/
--   https://docs.asterisk.org/Latest_API/.../CEL/
-- ============================================================================


-- ----------------------------------------------------------------------------
-- CDR — Call Detail Records
-- ----------------------------------------------------------------------------
-- One row per call (well, per "CDR event"). Logs when, who, how long,
-- and the disposition (answered, busy, no answer, failed).
--
-- The `uniqueid` column is the key that links a CDR to:
--   * the recording file on disk (filename pattern includes uniqueid)
--   * CEL events (cel.uniqueid matches cdr.uniqueid)
--   * AMI events (AMI events carry uniqueid in the Channel headers)

CREATE TABLE cdr (
    id                  BIGSERIAL PRIMARY KEY,

    -- Call timestamps
    calldate            TIMESTAMPTZ NOT NULL DEFAULT now(),
    start_time          TIMESTAMPTZ,
    answer_time         TIMESTAMPTZ,
    end_time            TIMESTAMPTZ,

    -- Parties
    clid                TEXT NOT NULL DEFAULT '',
    src                 TEXT NOT NULL DEFAULT '',
    dst                 TEXT NOT NULL DEFAULT '',
    dcontext            TEXT NOT NULL DEFAULT '',
    channel             TEXT NOT NULL DEFAULT '',
    dstchannel          TEXT NOT NULL DEFAULT '',
    lastapp             TEXT NOT NULL DEFAULT '',
    lastdata            TEXT NOT NULL DEFAULT '',

    -- Duration
    duration            INTEGER NOT NULL DEFAULT 0,
    billsec             INTEGER NOT NULL DEFAULT 0,

    -- Result
    disposition         TEXT NOT NULL DEFAULT '',   -- ANSWERED, NO ANSWER, BUSY, FAILED, CONGESTION
    amaflags            INTEGER NOT NULL DEFAULT 0,
    accountcode         TEXT NOT NULL DEFAULT '',

    -- Asterisk identifiers
    uniqueid            TEXT NOT NULL,
    userfield           TEXT NOT NULL DEFAULT '',

    -- Post-1.8 columns (linkedid is used for call grouping)
    peeraccount         TEXT NOT NULL DEFAULT '',
    linkedid            TEXT NOT NULL DEFAULT '',
    sequence            INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_cdr_calldate    ON cdr (calldate DESC);
CREATE INDEX idx_cdr_uniqueid    ON cdr (uniqueid);
CREATE INDEX idx_cdr_linkedid    ON cdr (linkedid);
CREATE INDEX idx_cdr_src         ON cdr (src);
CREATE INDEX idx_cdr_dst         ON cdr (dst);
CREATE INDEX idx_cdr_disposition ON cdr (disposition);


-- ----------------------------------------------------------------------------
-- CEL — Channel Event Logging
-- ----------------------------------------------------------------------------
-- One row per channel event. Many rows per call. Captures the full
-- lifecycle: CHAN_START, ANSWER, BRIDGE_ENTER, TRANSFER, HANGUP, etc.
--
-- The `eventtype` column is what distinguishes events. The `eventtime`
-- is when it happened. The `uniqueid` groups events within a single
-- channel; `linkedid` groups all channels belonging to the same call.

CREATE TABLE cel (
    id                  BIGSERIAL PRIMARY KEY,

    eventtype           TEXT NOT NULL,
    eventtime           TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Channel this event is about
    userdeftype         TEXT NOT NULL DEFAULT '',
    cid_name            TEXT NOT NULL DEFAULT '',
    cid_num             TEXT NOT NULL DEFAULT '',
    cid_ani             TEXT NOT NULL DEFAULT '',
    cid_rdnis           TEXT NOT NULL DEFAULT '',
    cid_dnid            TEXT NOT NULL DEFAULT '',

    exten               TEXT NOT NULL DEFAULT '',
    context             TEXT NOT NULL DEFAULT '',
    channame            TEXT NOT NULL DEFAULT '',
    appname             TEXT NOT NULL DEFAULT '',
    appdata             TEXT NOT NULL DEFAULT '',

    amaflags            INTEGER NOT NULL DEFAULT 0,
    accountcode         TEXT NOT NULL DEFAULT '',
    peer                TEXT NOT NULL DEFAULT '',
    userfield           TEXT NOT NULL DEFAULT '',

    -- Identifiers
    uniqueid            TEXT NOT NULL,
    linkedid            TEXT NOT NULL DEFAULT '',

    -- Additional JSON-encoded data (Asterisk 16+)
    extra               TEXT NOT NULL DEFAULT ''
);

CREATE INDEX idx_cel_eventtime ON cel (eventtime DESC);
CREATE INDEX idx_cel_uniqueid  ON cel (uniqueid);
CREATE INDEX idx_cel_linkedid  ON cel (linkedid);
CREATE INDEX idx_cel_eventtype ON cel (eventtype);
CREATE INDEX idx_cel_channame  ON cel (channame);
