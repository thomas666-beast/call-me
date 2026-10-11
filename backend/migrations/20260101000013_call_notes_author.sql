-- ============================================================================
-- Allow call_notes.author_id to become NULL when a user is deleted.
-- ============================================================================
--
-- Previously it was NOT NULL with ON DELETE CASCADE, which meant deleting
-- an agent also deleted every note they had written. That's wrong — the
-- note belongs to the call, not the author.

ALTER TABLE call_notes
    ALTER COLUMN author_id DROP NOT NULL;

ALTER TABLE call_notes
    DROP CONSTRAINT IF EXISTS call_notes_author_id_fkey;

ALTER TABLE call_notes
    ADD CONSTRAINT call_notes_author_id_fkey
        FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE SET NULL;
