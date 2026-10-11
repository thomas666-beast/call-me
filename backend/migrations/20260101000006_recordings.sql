-- ============================================================================
-- recordings — indexed call recording files
-- ============================================================================
--
-- Unlike cdr and cel, this table is written by OUR BACKEND, not Asterisk.
-- The backend scans the recording directory (a background job, to be added
-- in a later message), matches files to CDR rows by uniqueid, and populates
-- this table.
--
-- Asterisk writes recordings to /var/spool/asterisk/monitor/ by default,
-- in the format <uniqueid>.wav (or .gsm, .ulaw, .mp3 depending on config).
-- We configure MixMonitor to use WAV in the dialplan.
--
-- Multiple recordings can exist per call:
--   * Full call recording (both legs mixed)
--   * Per-agent recording (if the dialplan records each leg separately)
--   * Manual recordings triggered by an agent during a call
--
-- The `path` column is the absolute path on the HOST filesystem, so the
-- backend can serve the file directly. The mount ensures the same path
-- is valid both inside and outside the container.

CREATE TABLE recordings (
    id              UUID PRIMARY KEY,

    -- Links to cdr.uniqueid (the Asterisk-assigned channel id)
    uniqueid        TEXT NOT NULL,
    linkedid        TEXT NOT NULL DEFAULT '',

    -- File details
    path            TEXT NOT NULL,             -- absolute path on host
    filename        TEXT NOT NULL,             -- basename only
    format          TEXT NOT NULL,             -- 'wav', 'mp3', 'ogg', 'gsm'
    size_bytes      BIGINT NOT NULL DEFAULT 0,
    duration_secs   INTEGER,                   -- NULL until computed

    -- Call context (denormalized from cdr for quick listing)
    call_date       TIMESTAMPTZ,
    src             TEXT NOT NULL DEFAULT '',
    dst             TEXT NOT NULL DEFAULT '',
    queue_number    TEXT,                       -- if call came from a queue
    agent_extension TEXT,                       -- which agent handled it

    -- Indexing metadata
    indexed_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    file_mtime      TIMESTAMPTZ,                -- filesystem mtime, for change detection

    -- Playback audit (later phase)
    last_played_at  TIMESTAMPTZ,
    last_played_by  UUID REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_recordings_uniqueid  ON recordings (uniqueid);
CREATE INDEX idx_recordings_linkedid  ON recordings (linkedid);
CREATE INDEX idx_recordings_call_date ON recordings (call_date DESC);
CREATE INDEX idx_recordings_path      ON recordings (path);
