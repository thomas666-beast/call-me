-- Extensions: SIP endpoints managed by the admin UI.
--
-- Notes on design:
-- * `number` is the PJSIP endpoint name AND the dialplan extension.
-- * `secret` is stored in plaintext because Asterisk's PJSIP auth
--   requires the plaintext password to compute the digest at runtime.
--   In production, the Postgres volume must be encrypted at rest.
-- * `codecs` is a Postgres TEXT[] array. PJSIP expects a comma-separated
--   list; the config generator handles the conversion.
-- * `active = false` disables the extension without deleting it.

CREATE TABLE extensions (
    id             UUID PRIMARY KEY,
    number         TEXT UNIQUE NOT NULL,
    display_name   TEXT NOT NULL,
    secret         TEXT NOT NULL,
    context        TEXT NOT NULL DEFAULT 'from-internal',
    transport      TEXT NOT NULL DEFAULT 'transport-udp',
    codecs         TEXT[] NOT NULL DEFAULT ARRAY['ulaw','alaw']::TEXT[],
    max_contacts   INTEGER NOT NULL DEFAULT 1,
    active         BOOLEAN NOT NULL DEFAULT true,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_extensions_number ON extensions (number);
CREATE INDEX idx_extensions_active ON extensions (active);
