-- ============================================================================
-- seed-test-contacts.sql — fake contacts for testing
-- ============================================================================
--
-- Creates 20 fake "external" contacts. They don't belong to our organization
-- and don't have extensions. They exist only in the contacts table so the
-- agent panel has something to match incoming caller IDs against.
--
-- All contacts are tagged "test" so they can be bulk-deleted later:
--     DELETE FROM contacts WHERE 'test' = ANY(tags);
--
-- Run:
--     docker exec -i asterisk-pg psql -U callcenter -d callcenter \
--       < /home/thomas/Coding/Rust/call-me/scripts/seed-test-contacts.sql
-- ============================================================================

INSERT INTO contacts (id, phone_number, display_name, company, tags, notes)
VALUES
    (gen_random_uuid(), '+15551000001', 'Alice Johnson',  'Acme Inc',    ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000002', 'Bob Smith',      'Globex',      ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000003', 'Carol White',    'Initech',     ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000004', 'David Brown',    'Umbrella',    ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000005', 'Eva Martinez',   'Acme Inc',    ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000006', 'Frank Wilson',   'Soylent',     ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000007', 'Grace Lee',      'Globex',      ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000008', 'Henry Davis',    'Initech',     ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000009', 'Ivy Clark',      'Umbrella',    ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000010', 'Jack Taylor',    'Acme Inc',    ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000011', 'Kate Anderson',  'Cyberdyne',   ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000012', 'Liam Thomas',    'Soylent',     ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000013', 'Mia Jackson',    'Globex',      ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000014', 'Noah Harris',    'Initech',     ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000015', 'Olivia Martin',  'Umbrella',    ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000016', 'Peter Thompson', 'Cyberdyne',   ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000017', 'Quinn Garcia',   'Acme Inc',    ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000018', 'Rachel Rodriguez','Globex',     ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000019', 'Sam Lewis',      'Soylent',     ARRAY['test','client'], 'Test contact — fake inbound caller'),
    (gen_random_uuid(), '+15551000020', 'Tina Walker',    'Initech',     ARRAY['test','client'], 'Test contact — fake inbound caller')
ON CONFLICT (phone_number) DO NOTHING;

-- Report
SELECT count(*) AS test_contacts FROM contacts WHERE 'test' = ANY(tags);