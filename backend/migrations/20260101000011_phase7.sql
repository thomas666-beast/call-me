-- ============================================================================
-- Phase 7: call_notes + agent_status_log + permissions
-- ============================================================================

-- Multiple notes per call. Linked by uniqueid AND linkedid so we can find
-- notes regardless of which channel id the frontend has.
CREATE TABLE call_notes (
    id            UUID PRIMARY KEY,
    call_uniqueid TEXT NOT NULL,
    call_linkedid TEXT NOT NULL DEFAULT '',
    author_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    note          TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_call_notes_uniqueid ON call_notes (call_uniqueid);
CREATE INDEX idx_call_notes_linkedid ON call_notes (call_linkedid);
CREATE INDEX idx_call_notes_author   ON call_notes (author_id);
CREATE INDEX idx_call_notes_created  ON call_notes (created_at DESC);


-- Audit log of pause/unpause/join/leave actions with the reason code.
-- Asterisk stores the current state; this table stores the history + reason.
CREATE TABLE agent_status_log (
    id            BIGSERIAL PRIMARY KEY,
    extension     TEXT NOT NULL,
    queue_number  TEXT NOT NULL,
    event_type    TEXT NOT NULL,       -- 'pause', 'unpause', 'join', 'leave'
    reason        TEXT,                -- pause reason code; null for other events
    actor_id      UUID REFERENCES users(id) ON DELETE SET NULL,
    occurred_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_asl_extension_time ON agent_status_log (extension, occurred_at DESC);
CREATE INDEX idx_asl_queue_time     ON agent_status_log (queue_number, occurred_at DESC);


-- Permissions for call notes (agents.read / agents.write already exist).
INSERT INTO permissions (name, description) VALUES
    ('call_notes.read',  'View call notes'),
    ('call_notes.write', 'Create and edit call notes')
ON CONFLICT (name) DO NOTHING;

INSERT INTO role_permissions (role, permission_id)
SELECT 'admin', id FROM permissions WHERE name IN ('call_notes.read', 'call_notes.write')
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role, permission_id)
SELECT 'supervisor', id FROM permissions WHERE name IN ('call_notes.read', 'call_notes.write')
ON CONFLICT DO NOTHING;

INSERT INTO role_permissions (role, permission_id)
SELECT 'agent', id FROM permissions WHERE name IN ('call_notes.read', 'call_notes.write')
ON CONFLICT DO NOTHING;
