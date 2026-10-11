INSERT INTO permissions (name, description) VALUES
    ('users.read',    'View users'),
    ('users.write',   'Create/update/delete users'),
    ('queues.read',   'View queues'),
    ('queues.write',  'Create/update/delete queues'),
    ('agents.read',   'View agent state'),
    ('agents.write',  'Modify agent state'),
    ('cdr.read',      'View call logs'),
    ('recordings.read','Listen to recordings'),
    ('config.read',   'View Asterisk config'),
    ('config.write',  'Modify Asterisk config');

-- Admin: everything
INSERT INTO role_permissions (role, permission_id)
SELECT 'admin', id FROM permissions;

-- Supervisor: read-only across the board, plus agent control
INSERT INTO role_permissions (role, permission_id)
SELECT 'supervisor', id FROM permissions
WHERE name IN ('users.read', 'queues.read', 'agents.read', 'agents.write',
               'cdr.read', 'recordings.read', 'config.read');

-- Agent: minimal — own stats only
INSERT INTO role_permissions (role, permission_id)
SELECT 'agent', id FROM permissions
WHERE name IN ('queues.read', 'cdr.read');
