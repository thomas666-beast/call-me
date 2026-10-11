-- ============================================================================
-- contacts — external caller directory
-- ============================================================================
--
-- Stores information about people who call us. The `phone_number` column
-- is the key: when a call comes in with src = '+15551234567', we can look
-- up the contact by phone_number and show their name in the UI.
--
-- Phone numbers are stored as strings without formatting, normalized
-- on write (see routes/contacts.rs). This ensures the lookup works
-- regardless of how the number was entered.

CREATE TABLE contacts (
    id             UUID PRIMARY KEY,
    phone_number   TEXT UNIQUE NOT NULL,
    display_name   TEXT NOT NULL,
    email          TEXT,
    company        TEXT,
    notes          TEXT NOT NULL DEFAULT '',
    tags           TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    created_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_contacts_phone ON contacts (phone_number);
CREATE INDEX idx_contacts_name  ON contacts (display_name);

-- Full-text search across name, company, notes.
-- Using a simple tsvector expression index so that ILIKE queries
-- remain fast as the table grows.
CREATE INDEX idx_contacts_search ON contacts
    USING gin (to_tsvector('simple',
        coalesce(display_name, '') || ' ' ||
        coalesce(company, '') || ' ' ||
        coalesce(notes, '')
    ));


-- ============================================================================
-- New permissions for contacts
-- ============================================================================

INSERT INTO permissions (name, description) VALUES
    ('contacts.read',  'View caller directory'),
    ('contacts.write', 'Create/update/delete caller directory entries');

-- Admin and supervisor get full access.
-- Agents get read-only, so they can identify callers.
INSERT INTO role_permissions (role, permission_id)
SELECT 'admin', id FROM permissions WHERE name IN ('contacts.read', 'contacts.write');

INSERT INTO role_permissions (role, permission_id)
SELECT 'supervisor', id FROM permissions WHERE name IN ('contacts.read', 'contacts.write');

INSERT INTO role_permissions (role, permission_id)
SELECT 'agent', id FROM permissions WHERE name = 'contacts.read';
