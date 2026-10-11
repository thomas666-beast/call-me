-- ============================================================================
-- settings — key/value store for runtime configuration
-- ============================================================================
--
-- Simple key/value table for settings that admins can change without
-- restarting the backend. Values are stored as TEXT; typed accessors
-- live in models/setting.rs.

CREATE TABLE settings (
    key         TEXT PRIMARY KEY,
    value       TEXT NOT NULL,
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed the defaults so the app has something to read on first boot.
INSERT INTO settings (key, value) VALUES
    ('recordings.retention_days', '0')   -- 0 = keep forever
ON CONFLICT (key) DO NOTHING;
