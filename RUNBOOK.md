# Asterisk Manager — RUNBOOK

A web-based management interface for Asterisk PBX, built for small
call centers. Manages extensions, queues, agents, contacts, call logs,
and call recordings.

---

## Table of contents

1.  Introduction
2.  Prerequisites
3.  Project layout
4.  First-time setup
5.  Daily development workflow
6.  URLs, credentials, and ports
7.  Database operations
8.  Asterisk configuration
9.  Extensions
10. Queues
11. Agents and panel users
12. Call log (CDR) and call classification
13. Recordings
14. Contacts
15. Agent panel
16. Settings (retention, etc.)
17. Backups and restore
18. Security
19. Troubleshooting
20. Architecture notes
21. What is not built (roadmap)

---

## 1. Introduction

This system manages a single Asterisk PBX instance and provides:

- **Admin interface** — extensions, queues, agents, contacts, users,
  call logs, recordings, settings.
- **Agent panel** — queue availability, active call tabs, per-call
  notes, contact creation on the fly.
- **Call logging** — CDR records classified by kind (queue / internal /
  external / outbound) with outcome (answered by who / abandoned /
  missed / cancelled).
- **Recordings** — automatically indexed, searchable, playable with
  HTTP Range support, with configurable retention.

Three roles exist:

| Role | What they can do |
|---|---|
| `admin` | Everything — panel accounts, agents, extensions, queues, contacts, calls, settings |
| `supervisor` | Read-only access to most things + agent control (UI partially built — see §21) |
| `agent` | Log in to the Agent Panel only |

---

## 2. Prerequisites

| Tool | Version | Verify with |
|---|---|---|
| Rust | stable | `rustc --version` |
| Node.js | 20+ | `node --version` |
| Docker | any recent | `docker --version` |
| Docker Compose | v2 | `docker compose version` |
| `jq` | any | `jq --version` |
| `openssl` | any | `openssl version` |
| `nc` (netcat) | any | `nc -h` |

Optional but strongly recommended:

- A softphone: **MicroSIP** (Windows), **Linphone** (cross-platform),
  or **Zoiper** (mobile).

---

## 3. Project layout

    call-me/
    ├── docker-compose.yml
    ├── RUNBOOK.md                       # this file
    ├── asterisk-config/                 # mounted into Asterisk container
    │   ├── manager.conf                 # AMI users (static)
    │   ├── pjsip.conf                   # PJSIP transport (static)
    │   ├── pjsip_extensions.conf        # GENERATED — endpoints
    │   ├── extensions.conf              # dialplan (static)
    │   ├── queues.conf                  # GENERATED — queues
    │   ├── cdr_pgsql.conf               # CDR to Postgres
    │   ├── cel_pgsql.conf               # CEL to Postgres
    │   ├── cel.conf                     # CEL enabled
    │   ├── res_pgsql.conf               # Realtime driver for queue_log
    │   ├── extconfig.conf               # queue_log → pgsql mapping
    │   ├── modules.conf                 # module autoload config
    │   └── cdr.conf                     # CDR engine settings
    ├── asterisk-recordings/             # mounted as /var/spool/asterisk/monitor
    ├── asterisk-logs/                   # mounted as /var/log/asterisk
    ├── asterisk-backups/                # pg_dump output (git-ignored)
    ├── scripts/
    │   ├── backup-loop.sh               # runs in backup sidecar
    │   ├── restore-db.sh
    │   ├── generate-secrets.sh
    │   ├── simulate-call.sh             # test helper
    │   ├── seed-test-contacts.sql
    │   └── README.md
    ├── backend/                         # Rust
    │   ├── migrations/                  # sqlx SQL migrations
    │   ├── src/
    │   │   ├── ami/                     # AMI client + actions
    │   │   ├── auth/                    # JWT, RBAC, rate limit, signed URLs
    │   │   ├── config_gen/              # Asterisk config generation
    │   │   ├── db/                      # pool + seed
    │   │   ├── models/                  # DB row types
    │   │   ├── routes/                  # HTTP handlers
    │   │   ├── web/                     # WebSocket fan-out
    │   │   ├── recordings_indexer.rs
    │   │   ├── recordings_retention.rs
    │   │   └── main.rs
    │   ├── Cargo.toml
    │   ├── .env                         # real secrets (git-ignored)
    │   └── .env.example
    └── frontend/                        # Vue 3
        ├── src/
        │   ├── api/                     # typed API clients
        │   ├── components/
        │   ├── router/
        │   ├── stores/
        │   └── views/
        └── package.json

**Generated files — never edit by hand:**

- `asterisk-config/pjsip_extensions.conf`
- `asterisk-config/queues.conf`

The backend rewrites these on every extension/queue change.

---

## 4. First-time setup

### 4.1 Clone and generate secrets

    cd /home/thomas/Coding/Rust/call-me
    ./scripts/generate-secrets.sh

You get three values: `JWT_SECRET`, `PGPASSWORD`, `AMI_PASSWORD`. Keep them.

### 4.2 Fill in configuration

- **`backend/.env`** — copy from `.env.example` and paste the new
  secrets. Set `APP_ENV=development` for local use.
- **`asterisk-config/manager.conf`** — paste `AMI_PASSWORD` as the
  `secret =` value for the `[rustapp]` user.
- **`docker-compose.yml`** — paste `PGPASSWORD` in **three places**:
  the `asterisk` service's `PGPASSWORD` env, the `postgres` service's
  `POSTGRES_PASSWORD`, and the `backup` service's `PGPASSWORD`.

### 4.3 Create host directories

    mkdir -p asterisk-recordings asterisk-logs asterisk-backups
    chmod 777 asterisk-recordings asterisk-logs
    chmod 755 asterisk-backups

### 4.4 Start the containers

    docker compose up -d
    docker compose ps

Wait for `asterisk-pg` to show `healthy`. Asterisk takes ~10s to be ready.

### 4.5 Build and run the backend

    cd backend
    cargo build
    cargo clippy --all-targets -- -D warnings
    cargo run

Watch for:

    INFO asterisk_manager: Starting Asterisk Manager (env: development)
    INFO asterisk_manager::db::pool: Connected to PostgreSQL
    INFO asterisk_manager::db::pool: Migrations applied
    INFO asterisk_manager::ami::client: AMI login response: "...Success..."
    INFO asterisk_manager::web: HTTP/WebSocket listening on 0.0.0.0:8080

### 4.6 Run the frontend

    cd ../frontend
    npm install
    npm run dev

Open `http://localhost:5173`. Log in as `admin / admin` (dev seed — see §4.7).

### 4.7 Dev users

When `SEED_DEV_USERS=true` in `backend/.env`, two users are created on
first start:

| Username | Password | Role | Extension |
|---|---|---|---|
| `admin`   | `admin`   | admin | — |
| `agent1`  | `agent1`  | agent | 1001 |

**Change these before exposing the system.** They exist to make first
boot painless.

---

## 5. Daily development workflow

Three terminals:

**Terminal 1 — containers (start once):**

    cd /home/thomas/Coding/Rust/call-me
    docker compose up -d

**Terminal 2 — backend:**

    cd backend
    cargo run

**Terminal 3 — frontend:**

    cd frontend
    npm run dev

**Stopping:**

- `Ctrl+C` in terminals 2 and 3.
- `docker compose down` in terminal 1 (or just leave it running).

**Rebuilding after backend changes:**

- `cargo build` then `Ctrl+C` and `cargo run` again.
- If you change a migration file, rebuild — `sqlx::migrate!` reads the
  migrations directory at **compile time**, not runtime.

---

## 6. URLs, credentials, and ports

| Service | URL / port |
|---|---|
| Frontend | http://localhost:5173 |
| Backend API | http://localhost:8080 |
| Health check | http://localhost:8080/health |
| AMI (raw) | tcp://localhost:5038 |
| Postgres | localhost:5432 |
| SIP (UDP/TCP) | 5060 |
| RTP media | 10000–10100 UDP (mapped) |

**Postgres credentials** are whatever you generated in §4.1.

**AMI credentials** — `rustapp` / (your generated password).

**Frontend ↔ backend** — the base URL is hardcoded in
`frontend/src/api/client.ts` as `http://localhost:8080`.

---

## 7. Database operations

### Connect to the database

    docker exec -it asterisk-pg psql -U callcenter -d callcenter

### Useful queries

    -- Users (all roles)
    SELECT username, role, extension, active FROM users ORDER BY username;

    -- Panel users only
    SELECT username, role, email, active FROM users
     WHERE role IN ('admin', 'supervisor') ORDER BY username;

    -- Extensions
    SELECT number, display_name, active FROM extensions ORDER BY number;

    -- Queues
    SELECT number, name, active FROM queues ORDER BY number;

    -- Queue membership
    SELECT q.number, e.number AS ext
      FROM queue_members qm
      JOIN queues q ON q.id = qm.queue_id
      JOIN extensions e ON e.id = qm.extension_id
     ORDER BY q.number, e.number;

    -- Contacts
    SELECT phone_number, display_name, company FROM contacts ORDER BY display_name;

    -- Recent calls
    SELECT calldate, src, dst, disposition, linkedid
      FROM cdr ORDER BY calldate DESC LIMIT 20;

    -- Recent queue events
    SELECT "time", callid, queuename, agent, event
      FROM queue_log ORDER BY "time" DESC LIMIT 20;

    -- Recent recordings
    SELECT filename, size_bytes, src, dst
      FROM recordings ORDER BY indexed_at DESC LIMIT 20;

    -- Recent login attempts
    SELECT username, success, created_at
      FROM auth_audit_log ORDER BY created_at DESC LIMIT 20;

    -- Agent status log (pause/resume)
    SELECT extension, queue_number, event_type, reason, occurred_at
      FROM agent_status_log ORDER BY occurred_at DESC LIMIT 20;

### Migrations

Migrations live in `backend/migrations/`. They are numbered
`YYYYMMDDNNNNNN_description.sql`.

**To add a new migration:**

1. Create the file with a **newer** timestamp than any existing one.
2. **`cargo build`** — the `sqlx::migrate!` macro reads the directory at
   compile time and embeds the list into the binary.
3. `cargo run` — pending migrations apply on startup.

**Never edit a migration that has already run on any machine.**
`sqlx` stores a checksum of each file and refuses to start if it
changes. If you need to modify a table, create a new migration.

### Rotating the Postgres password

Changing the Postgres password is a **five-file** operation. Missing any
one of them causes silent failures — for example, Asterisk's CDR engine
will keep trying the old password and log nothing for hours while the
system looks healthy.

**The five places that hold the password:**

| # | File | Key |
|---|---|---|
| 1 | `docker-compose.yml` → `postgres` service | `POSTGRES_PASSWORD` |
| 2 | `docker-compose.yml` → `asterisk` service | `PGPASSWORD` |
| 3 | `docker-compose.yml` → `backup` service | `PGPASSWORD` |
| 4 | `backend/.env` | password inside `DATABASE_URL` |
| 5 | `asterisk-config/cdr_pgsql.conf` | `password =` |
| 6 | `asterisk-config/cel_pgsql.conf` | `password =` |
| 7 | `asterisk-config/res_pgsql.conf` | `password =` |

(That's actually 7 entries across 5 files — three of them are inside
`docker-compose.yml` and three are inside `asterisk-config/`.)

**Correct procedure, in order:**

1. **Generate the new password:**

       openssl rand -hex 16

2. **Change it in the running database first:**

       docker exec -it asterisk-pg psql -U callcenter -d callcenter \
         -c "ALTER USER callcenter WITH PASSWORD '<NEW_PASSWORD>';"

3. **Update `docker-compose.yml`** — three `PGPASSWORD` / `POSTGRES_PASSWORD` values.

4. **Update `backend/.env`** — the password inside `DATABASE_URL`.

5. **Update the three Asterisk config files:**
   - `asterisk-config/cdr_pgsql.conf`
   - `asterisk-config/cel_pgsql.conf`
   - `asterisk-config/res_pgsql.conf`

6. **Reload Asterisk modules:**

       docker exec asterisk-lab asterisk -rx "module reload cdr_pgsql.so"
       docker exec asterisk-lab asterisk -rx "module reload cel_pgsql.so"
       docker exec asterisk-lab asterisk -rx "module reload res_config_pgsql.so"

7. **Restart the backend** (`Ctrl+C` then `cargo run`) so it picks up
   the new `DATABASE_URL`.

8. **Restart the backup sidecar:**

       docker compose up -d backup

9. **Verify:**

       # Backend connects
       cargo run
       # → "Connected to PostgreSQL" in the log

       # Asterisk writes CDRs
       # Make a test call, then:
       docker exec -it asterisk-pg psql -U callcenter -d callcenter -c \
         "SELECT id, calldate, src, dst FROM cdr ORDER BY id DESC LIMIT 1;"

       # Backup runs
       docker compose up -d backup
       docker logs asterisk-pg-backup --tail 5

**Symptom of a partial rotation:** Asterisk works for SIP and dialplan,
the backend works, but no CDRs or CEL events are written. The Asterisk
log shows:

    cdr_pgsql.c: pgsql_log: Reason: connection to server at "postgres" ...
    password authentication failed for user "callcenter"

If you see this, steps 5–6 above are what's missing.


### Reset the database

    docker compose down -v        # -v removes the pg_data volume
    docker compose up -d
    # Next `cargo run` re-runs all migrations and re-seeds dev users.

This destroys everything — users, contacts, CDRs, recordings metadata.

---

## 8. Asterisk configuration

### Mounted files

Everything in `asterisk-config/` on the host is mounted at
`/etc/asterisk/` inside the container. This is the correct pattern —
**do not** mount individual files (see §20 for why).

### Static vs generated

| File | Managed by |
|---|---|
| `manager.conf` | hand-edited |
| `pjsip.conf` | hand-edited (defines transport) |
| `extensions.conf` | hand-edited (dialplan) |
| `cdr_pgsql.conf`, `cel_pgsql.conf`, `cel.conf` | hand-edited |
| `res_pgsql.conf`, `extconfig.conf`, `modules.conf` | hand-edited |
| `pjsip_extensions.conf` | **generated by backend** |
| `queues.conf` | **generated by backend** |

### Reloading Asterisk

The backend reloads automatically after every write:

- Extension change → `module reload res_pjsip.so`
- Queue change → `QueueReload` action + `dialplan reload`
- Agent change → both of the above

If you edit `manager.conf` or `pjsip.conf` by hand, reload manually:

    docker exec asterisk-lab asterisk -rx "manager reload"
    docker exec asterisk-lab asterisk -rx "module reload res_pjsip.so"

### Inspecting Asterisk live

    # All PJSIP endpoints
    docker exec asterisk-lab asterisk -rx "pjsip show endpoints"

    # All queues and members
    docker exec asterisk-lab asterisk -rx "queue show"

    # A specific queue
    docker exec asterisk-lab asterisk -rx "queue show 2000"

    # Active channels
    docker exec asterisk-lab asterisk -rx "core show channels"

    # AMI users connected
    docker exec asterisk-lab asterisk -rx "manager show connected"

    # Which CDR/CEL modules are loaded
    docker exec asterisk-lab asterisk -rx "module show like cdr"
    docker exec asterisk-lab asterisk -rx "module show like cel"

### Testing AMI directly (the most useful debugging tool)

When something AMI-related misbehaves, look at the raw bytes:

    nc 127.0.0.1 5038

Then paste:

    Action: Login
    Username: rustapp
    Secret: <your AMI password>
    Events: off

    Action: Command
    ActionID: manual-1
    Command: module reload res_pjsip.so

Expected response (Asterisk 22):

    Response: Success
    ActionID: manual-1
    Message: Command output follows
    Output: Module 'res_pjsip.so' reloaded successfully.

**No `--END COMMAND--` marker in Asterisk 22** for short-output commands.
The parser in `backend/src/ami/actions.rs` handles both shapes.

`Ctrl+C` to exit.

### Container logs

    docker logs asterisk-lab --tail 100
    docker logs asterisk-pg --tail 50
    docker logs asterisk-pg-backup --tail 30
    docker logs -f asterisk-lab      # follow mode

### Asterisk 22 quirks to remember

- `Command: module unload app_queue.so` is **blacklisted** via AMI.
  Use the dedicated `QueueReload` action instead.
- `queues.conf` is **not re-read** by `module reload app_queue.so` or
  `queue reload all`. `QueueReload` is the correct action.
- `res_config_pgsql` must be loaded for `queue_log` realtime writes.
  The mapping is in `extconfig.conf`.

---

## 9. Extensions

### Via the UI

Admin → **Extensions** → **New extension**.

Fields:

| Field | Notes |
|---|---|
| Number | 2–10 digits. Convention: start with `1` (e.g. `1001`) |
| Display name | Human-readable name |
| SIP secret | Password for the softphone. Min 6 chars |
| Context | Usually `from-internal` |
| Codecs | `ulaw`, `alaw` are the safe defaults |
| Max contacts | Usually `1` (one device per extension) |
| Active | Disable to keep the config but stop serving it |

After save, the backend regenerates `pjsip_extensions.conf` and reloads
PJSIP. The extension appears in `pjsip show endpoints`.

### Via the API

    TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
      -H "Content-Type: application/json" \
      -d '{"username":"admin","password":"admin"}' | jq -r .access_token)

    curl -s -X POST http://localhost:8080/api/admin/extensions \
      -H "Authorization: Bearer $TOKEN" \
      -H "Content-Type: application/json" \
      -d '{"number":"1005","display_name":"Test","secret":"secret123"}'

### Softphone setup

| Field | Value |
|---|---|
| Server | Your host's LAN IP (**not** `localhost`) |
| Port | 5060 |
| Transport | UDP |
| Username | The extension number |
| Password | The SIP secret |
| Display name | Any |

After registering, `pjsip show endpoints` shows the extension as `Avail`.

---

## 10. Queues

### Via the UI

Admin → **Queues** → **New queue**.

| Field | Notes |
|---|---|
| Number | 4 digits, starts with `2` (e.g. `2000`) |
| Name | Human-readable |
| Strategy | `ringall`, `leastrecent`, `fewestcalls`, `random`, `rrmemory`, `rrordered`, `linear`, `wrandom` |
| Timeout | Seconds to ring a member before moving on |
| Retry | Seconds between rounds |
| Max waiting | Max callers in the queue |
| ACW | After-call work time. **Not enforced yet** (see §21) |
| Music class | Leave as `default` for now |
| Join empty | Accept callers even with no members |
| Leave when empty | Kick callers if all members leave |
| Ring in use | Ring agents already on a call |
| Members | Checkboxes for which extensions belong |

After save, the backend regenerates `queues.conf`, runs `QueueReload`,
and reloads the dialplan.

### Testing a queue

Softphone A dials `2000`. If the queue has members who are available,
all available members ring (with `ringall`). First to answer gets the
call.

Watch the queue in real time:

    watch -n 1 'docker exec asterisk-lab asterisk -rx "queue show 2000"'

### Simulate an inbound call

    ./scripts/simulate-call.sh                      # random fake number, queue 2000
    ./scripts/simulate-call.sh +15551000007         # specific number
    ./scripts/simulate-call.sh +15551000007 2001    # specific number, specific queue

Answer on a softphone. Hang up. Check `queue show` and the Calls page.

### Queue numbering convention

- **Extensions:** `1000`–`1999`
- **Queues:** `2000`–`2999`
- **Feature codes:** `*XX` (`*43` = echo test)

The backend rejects queue numbers that don't start with `2` and aren't
4 digits. This is enforced in `routes/queues.rs::validate()`.

---

## 11. Agents and panel users

Two different concepts, two different menus.

### Agents (people who take calls)

Admin → **Agents**.

An agent has:
- A **login username**, which is the same as their extension number.
- A **login password** for the web panel.
- An **SIP secret** for their softphone.
- One or more **queues** they belong to.

**Creating an agent:**

1. Click **New agent**.
2. Fill:
   - Extension number (e.g. `1099`) — becomes the login username.
   - Display name.
   - SIP secret — click **Generate** if unsure.
   - Login password — click **Generate** if unsure.
   - Queues — check the ones they belong to.
   - Active.
3. Click **Create**.
4. **A credentials modal appears** with the login password and SIP
   secret. **Copy them now** — they are not stored in plaintext and
   cannot be viewed again.

**If you lose the password:** click **Reset password** in the agent row.
A new one is generated and shown once.

**Deactivating an agent:** disables both the user account and the
extension. Historical calls and notes are preserved (soft-delete).

### Panel users (admins and supervisors)

Admin → **Users**.

These are accounts that can log into the panel but do **not** take
calls. They have no extension and no queues.

| Role | Purpose |
|---|---|
| `admin` | Full access to everything |
| `supervisor` | Currently read-only; supervisor-specific views are pending (see §21) |

**Creating a user:**

1. Click **New user**.
2. Fill username, email (optional), password, role.
3. Click **Create**.

The last active admin **cannot** be deactivated. This is enforced
server-side.

---

## 12. Call log (CDR) and call classification

Admin → **Calls**.

### What you see

One row per call. Calls are deduplicated by Asterisk's `linkedid` (the
internal identifier that ties together all channels belonging to the
same call).

| Column | Meaning |
|---|---|
| Date / Time | When the call started |
| Kind | `QUEUE`, `INTERNAL`, `EXTERNAL`, or `OUTBOUND` |
| Caller | Who initiated. Name is looked up from contacts/extensions if known |
| Destination | What they dialed |
| Outcome | For queue calls: `Answered by X (ext)`, `Abandoned`, `Missed`, `Cancelled`, `Timed out` |
| Talk | Billable seconds |
| Details | Opens the call detail page |

**Internal calls (extension → extension) are hidden by default.** Use
the "Show internal calls" checkbox to include them.

### How classification works

The backend classifies each call:

- **queue** — `dst` matches a queue number
- **internal** — `src` and `dst` both match extension numbers
- **outbound** — `src` matches an extension, `dst` doesn't
- **external** — neither end is ours

The outcome is derived from `queue_log`:

| Events | Outcome |
|---|---|
| has `CONNECT` | `answered:<extension>` |
| has `ABANDON` | `abandoned` |
| has `RINGCANCELED` | `cancelled` |
| has `EXITWITHTIMEOUT` | `timed_out` |
| has `RINGNOANSWER` | `missed` |
| none of the above | null |

### Call detail page

`Admin → Calls → Details` on any row.

Shows:
- **Banner** — the outcome (green for answered, red for abandoned, etc.)
- **Detail card** — all CDR fields + who answered
- **Queue events timeline** — every `ENTERQUEUE`, `CONNECT`, etc. from `queue_log`
- **Recording player** — if a recording exists
- **CEL timeline** — raw channel events

### Filtering

Filters: date range, caller, destination, disposition.
"Show internal calls" checkbox toggles internal calls.

---

## 13. Recordings

### How recordings are created

The dialplan (`extensions.conf`) adds `MixMonitor` to every call:

    exten => _1XXX,1,MixMonitor(${UNIQUEID}.wav,b)

`b` means "only record when bridged" (no silent files). Recordings
land in `/var/spool/asterisk/monitor/` inside the container, which is
bind-mounted to `asterisk-recordings/` on the host.

The filename is the Asterisk `UNIQUEID`. The backend's **recordings
indexer** scans this directory every 60 seconds, matches each new file
to a CDR row by `uniqueid` or `linkedid`, and inserts a row in the
`recordings` table.

### Playing recordings

Open a call detail page. If a recording exists, an audio player appears.
The player:

1. Requests a short-lived signed URL from the backend
   (`POST /api/admin/recordings/:id/play-token`).
2. Uses that URL as `<audio src=...>`.

The URL is valid for 5 minutes and cannot be reused for a different
recording. This is deliberate — JWTs in URLs leak into browser history
and logs, so we use one-time signed URLs instead.

### Retention

Admin → **Settings** → **Recordings retention**.

Presets: keep forever, 30 days, 90 days, 180 days, 1 year, 2 years.

The backend runs a daily task (`recordings_retention`) that deletes:
1. The file on disk.
2. The row in the `recordings` table.

You can also click **Run cleanup now** to apply the policy immediately.

**Retention defaults to "keep forever".** Set it to something sensible
before production.

### Manually cleaning up

    curl -s -X POST http://localhost:8080/api/admin/recordings/cleanup \
      -H "Authorization: Bearer $TOKEN" | jq

Returns `{ "deleted": N, "skipped": M, "retention_days": X }`.

---

## 14. Contacts

Admin → **Contacts**.

A shared directory of external callers. Agents can read and create
contacts. Admins can read, create, edit, and delete.

### Fields

- **Phone number** (normalized on save — `+15551234567`)
- **Display name**
- **Company**
- **Email**
- **Tags** (comma-separated)
- **Notes**

### How contacts are used

When a call arrives with a caller ID that matches a contact's phone
number, the agent panel shows the contact's name instead of the raw
number. The call detail page also shows the contact name.

### Creating from an agent panel

While on a call, agents see a "Create contact" button if the caller
isn't in the directory. The phone number is pre-filled.

### Search

Search matches against name, company, notes, and phone (partial match
on digits). Searching `555` finds `+15551234567`.

### Test contacts

`scripts/seed-test-contacts.sql` inserts 20 fake contacts
(`+15551000001` through `+15551000020`) tagged `test`. Useful for
simulating inbound calls without real external numbers.

    docker exec -i asterisk-pg psql -U callcenter -d callcenter \
      < scripts/seed-test-contacts.sql

Remove them later:

    docker exec -it asterisk-pg psql -U callcenter -d callcenter -c \
      "DELETE FROM contacts WHERE 'test' = ANY(tags);"

---

## 15. Agent panel

Agents log in at the same URL as admins, but land on `/agent/calls`.

### Three menu items

**My Panel** (`/agent`)
- The queues the agent belongs to, with pause/resume.
- The agent's own call history.

**Received Calls** (`/agent/calls`)
- The tabbed call interface.
- When a call comes in, a tab opens automatically with the caller's
  info (or a "create contact" form).
- Tabs stay open until the agent closes them manually.
- Tabs persist across navigation within the app.
- If the same caller calls again, the existing tab is reused — the new
  call's log replaces the old draft.

**Contacts** (`/agent/contacts`)
- Search any contact in the directory.
- View contact details and their call history.
- Create a new contact.
- Add a log to any past call.

### Queue availability

Agents can pause/resume each queue independently. Reasons:
`break`, `lunch`, `meeting`, `training`, `wrap-up`, `other`.

A paused agent is skipped by the queue. Pausing in one queue does not
affect others.

### Call log

Each call tab has a text area for notes. `Save log` saves the note
against the current call. The note becomes visible on the call detail
page in admin, tagged with the author and timestamp.

### Badge

A red badge appears on the "Received Calls" nav item when a new call
arrives while the agent is on another page.

---

## 16. Settings

Admin → **Settings**.

Currently only one setting: **recordings retention** (see §13).

The settings table (`settings`) is a key/value store. To add a new
setting:

1. Insert a default row in a migration.
2. Add read/write accessors in `models/setting.rs`.
3. Expose it in `routes/settings.rs`.
4. Add it to the frontend form.

---

## 17. Backups and restore

### How backups work

A sidecar container (`asterisk-pg-backup`) runs `pg_dump` against the
`callcenter` database:

- **On startup** — after a 20-second delay for Postgres readiness.
- **Every 24 hours** thereafter.

Backups land in `asterisk-backups/` on the host as
`callcenter-YYYYMMDD-HHMMSS.sql.gz`. Files older than 30 days are
deleted automatically.

### Configuration

In `docker-compose.yml`, under the `backup` service:

    BACKUP_INTERVAL_HOURS: "24"
    BACKUP_RETENTION_DAYS: "30"

Change and apply:

    docker compose up -d backup

### Verify backups are running

    docker ps | grep asterisk-pg-backup
    docker logs asterisk-pg-backup --tail 30
    ls -lh asterisk-backups/

### Run a backup right now

    docker exec asterisk-pg-backup sh -c \
      'pg_dump --no-owner --no-privileges | gzip > /backups/callcenter-manual-$(date -u +%Y%m%d-%H%M%S).sql.gz'

### Restore a backup

**Stop the backend first** (`Ctrl+C` in the `cargo run` terminal).

    cd /home/thomas/Coding/Rust/call-me
    ./scripts/restore-db.sh asterisk-backups/callcenter-20260926-120000.sql.gz

The script:
1. Warns and asks for confirmation.
2. Drops and recreates the `public` schema.
3. Loads the backup.

Then restart the backend:

    cd backend
    cargo run

### Off-host copies

**A backup on the same disk as the DB is not a backup.** Copy them
elsewhere regularly:

    rsync -av --delete asterisk-backups/ user@backup-host:/path/to/callcenter-backups/

Any off-host copy is fine — the point is one exists.

### When to test a restore

- **Before going to production.**
- **After any schema change.**
- **Every few months** as a drill.

A backup you've never restored is a backup you can't trust.

---

## 18. Security

### Secrets

- `JWT_SECRET` — used to sign access tokens. **Must be at least 32 chars.**
- Postgres password.
- AMI password (`rustapp`).

Generated by `scripts/generate-secrets.sh`. Stored in `.env` (git-ignored)
and `docker-compose.yml` (which you should also not commit if you include
real passwords — use `.env` on the CI/deploy side).

The backend **refuses to start** outside of `APP_ENV=development` if it
detects a default or short secret. See `config.rs::validate_secrets()`.

**To rotate the Postgres password**, follow the full procedure in
§7 — it touches 7 entries across 5 files. Skipping Asterisk's config
files causes silent CDR/CEL failures.

### Authentication

- **Access tokens** — JWT, 15-minute lifetime.
- **Refresh tokens** — opaque random 32 bytes, hashed (SHA-256) before
  storage. 7-day lifetime. Revoked on logout.
- **Passwords** — Argon2id.

### Rate limiting

Login attempts are limited to **10 per minute per IP**. On the 11th
attempt within the window, `429 Too Many Requests`.

A successful login clears the counter for that IP.

The cleanup task removes stale entries every 60 seconds.

### Audio playback URLs

Signed URLs (HMAC-SHA256 with a distinct key derived from `JWT_SECRET`)
valid for 5 minutes. No JWT in the URL, so nothing leaks to logs or
browser history. See `auth/signed_url.rs`.

### RBAC

Roles and permissions are in the database (`role_permissions` table).
Permission checks are done via typed extractors on handler signatures:

    CanReadUsers, CanWriteUsers, CanReadConfig, CanWriteConfig,
    CanReadQueues, CanWriteQueues, CanReadCdr, CanReadRecordings,
    CanReadContacts, CanWriteContacts, CanReadAgents, CanWriteAgents

### Before exposing the panel to the internet

- [ ] Rotate **all** secrets (see §4.1)
- [ ] Set `APP_ENV=production` in `.env`
- [ ] Set `SEED_DEV_USERS=false`
- [ ] Put a reverse proxy in front (nginx/Caddy) for **HTTPS**
- [ ] Change the dev passwords for `admin` and `agent1`
- [ ] Set recordings retention to something other than "forever"
- [ ] Confirm backups are running and copied off-host
- [ ] Test a restore drill

---

## 19. Troubleshooting

### Backend won't start

    # Port 8080 in use?
    lsof -i :8080

    # Postgres reachable?
    docker compose ps
    docker logs asterisk-pg --tail 30

    # AMI reachable?
    docker exec asterisk-lab asterisk -rx "manager show connected"

    # Migration mismatch?
    # "migration NNN was previously applied but has been modified"
    # — you edited a migration that already ran. Revert the edit, or
    # update the checksum (see §7).

### Login fails

    # Check audit log
    docker exec -it asterisk-pg psql -U callcenter -d callcenter -c \
      "SELECT username, success, created_at FROM auth_audit_log
       ORDER BY created_at DESC LIMIT 10;"

    # Verify user exists and is active
    docker exec -it asterisk-pg psql -U callcenter -d callcenter -c \
      "SELECT username, role, active FROM users WHERE username = 'admin';"

    # Rate limited?
    # Wait 60 seconds or restart the backend (clears in-memory limiter).

### 403 Forbidden on an endpoint

The role lacks the permission. Check:

    docker exec -it asterisk-pg psql -U callcenter -d callcenter -c \
      "SELECT rp.role, p.name
         FROM role_permissions rp
         JOIN permissions p ON p.id = rp.permission_id
        WHERE p.name = 'config.write';"

### Extension registered but no audio

- RTP range 10000–10100 UDP must be exposed in `docker-compose.yml`
  and allowed by the host firewall.
- Softphones must **not** use `localhost` for the server — use the
  host's LAN IP.

### Queue call: `dstchannel` is empty

The caller reached the queue, but no agent answered. Check:

    docker exec asterisk-lab asterisk -rx "queue show 2000"

Are members listed as `Not in use` or `Unavailable`? If `Unavailable`,
the softphone isn't registered.

### Recording file exists, but no row in `recordings` table

The indexer runs every 60 seconds. Wait a minute, then check the
backend log:

    # Should see: "Indexed recording: <filename> (<size> bytes)"

If not, the filename doesn't match a CDR's `uniqueid` or `linkedid`.
Look at the file name and search `cdr` for that id.

### `queue_log` table is empty

`res_config_pgsql` must be loaded and `extconfig.conf` must map
`queue_log => pgsql,callcenter`. Verify:

    docker exec asterisk-lab asterisk -rx "module show like res_config_pgsql"
    docker exec asterisk-lab asterisk -rx "config show help res_config_pgsql"

Reload if needed:

    docker exec asterisk-lab asterisk -rx "module reload res_config_pgsql.so"

### Asterisk container exits on start

    docker logs asterisk-lab --tail 50

Usually a syntax error in a mounted config file. Fix the file and:

    docker compose restart asterisk

### Frontend can't reach backend

CORS is wide-open in dev (any origin). If the browser console shows
CORS errors, check:

- Backend running on port 8080? `curl http://localhost:8080/health`
- Frontend base URL hardcoded to `http://localhost:8080` in
  `frontend/src/api/client.ts`

### Call detail page shows "No queue events"

The `linkedid` from the CDR didn't match any row in `queue_log`. This
can happen if the CDR was written before `res_config_pgsql` started
writing to `queue_log`. Check `queue_log` for that callid.

---

## 20. Architecture notes

### Request flow for an extension write

1. HTTP request → axum router → `require_auth` middleware
2. Handler runs `CanWriteConfig` extractor (checks `role_permissions`)
3. `sqlx` INSERT/UPDATE/DELETE
4. `regenerate_and_reload()`:
   a. SELECT all extensions
   b. Render `pjsip_extensions.conf` in memory
   c. Write temp file, atomic rename over target
   d. Run `module reload res_pjsip.so` via AMI
   e. On failure, restore `.bak` and retry
5. Return JSON

### Why directory mount (not individual files)

The Asterisk container mounts `asterisk-config/` as a **directory**.
Do not switch to mounting individual config files.

Single-file bind mounts pin the **inode** at container-start time.
When the backend replaces a file via `rename()` (which is atomic and
creates a new inode), the container's mount still points at the old
inode and never sees the update. A directory mount tracks the
directory inode, so file replacement propagates correctly.

### Why spawn + select + abort for AMI timeouts

`tokio::time::timeout` only cancels a future at its next `.await`
point. On some platforms, socket I/O can hold a future without
yielding, defeating the timeout. The `run_command` pattern is:

    let handle = tokio::spawn(async { ... });
    tokio::select! {
        res = &mut handle => ...,
        _ = tokio::time::sleep(TIMEOUT) => handle.abort(),
    }

This is the correct pattern for a hard timeout on blocking I/O.

### Why signed URLs for audio

The browser's `<audio>` element cannot send a custom Authorization
header. Putting the JWT in the query string leaks it into logs and
history. Signed URLs (HMAC over recording id + expiry) solve both:
one URL = one file = 5-minute window.

### Why dedupe by `linkedid`

Asterisk writes one CDR per channel. A queue call has two channels
(caller and agent), so two CDRs. The list endpoint uses
`DISTINCT ON (linkedid)` and prefers the row whose `src` is **not**
an extension (the caller's leg). This gives one row per call.

### Why `queue_log` realtime

CEL wasn't designed for queue events. CDR only logs completed calls.
AMI events are transient. `queue_log` is the only durable, complete
record of queue activity. Writing it directly to Postgres via
`res_config_pgsql` is cleaner than tailing a file.

---

## 21. What is not built (roadmap)

These are known gaps. They are not bugs; they're deferred features.

| # | Item | Notes |
|---|---|---|
| 1 | **Supervisor view** | The role exists and can log in. It has no dedicated UI. Either build it (see §11) or remove the role. |
| 2 | **Reports** | No UI for "calls this week", "avg talk time per agent", "queue SLA". Data is in `cdr` + `queue_log`. |
| 3 | **Production deployment guide** | Reverse proxy, systemd, static frontend build. See §18 for the pre-flight checklist. |
| 4 | **ACW enforcement** | `acw_seconds` is stored per queue but not enforced. Would pause the agent in that queue for N seconds after `AgentComplete`. |
| 5 | **Click-to-dial / call control** | No `Originate` from the panel. Agents use their softphones. |
| 6 | **Whisper / barge / listen** | No supervisor live-call monitoring. |
| 7 | **Wallboards** | No real-time queue/agent dashboard for supervisors. |
| 8 | **MP3 conversion** | Recordings stay as WAV. Larger files. `ffmpeg` conversion deferred. |
| 9 | **Multi-tenant** | Single Asterisk, single Postgres, single tenant. |
| 10 | **HA / clustering** | Backend is single-instance. No leader election, no state replication. |
| 11 | **WebRTC softphone in-browser** | Requires SIP-over-WSS, STUN/TURN. Separate project. |
| 12 | **Outbound calling** | No trunk. Extensions can only call each other and queues. |

### Known limitations to document for users

- Backend restart loses open agent tabs (data is safe; the tabs are
  not persisted across reloads).
- Internal calls between extensions are hidden by default in the call
  log.
- Asterisk restart is required to pick up changes to `manager.conf`
  or `pjsip.conf`.

---

## Appendix A — Quick reference

    # Health
    curl http://localhost:8080/health

    # Login
    TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
      -H "Content-Type: application/json" \
      -d '{"username":"admin","password":"admin"}' | jq -r .access_token)

    # Current user
    curl -s http://localhost:8080/api/auth/me \
      -H "Authorization: Bearer $TOKEN" | jq

    # List extensions
    curl -s http://localhost:8080/api/admin/extensions \
      -H "Authorization: Bearer $TOKEN" | jq

    # List queues
    curl -s http://localhost:8080/api/admin/queues \
      -H "Authorization: Bearer $TOKEN" | jq

    # List agents
    curl -s http://localhost:8080/api/admin/agents \
      -H "Authorization: Bearer $TOKEN" | jq

    # Recent calls (no internal)
    curl -s "http://localhost:8080/api/admin/cdr?limit=10" \
      -H "Authorization: Bearer $TOKEN" | jq

    # Recent calls (with internal)
    curl -s "http://localhost:8080/api/admin/cdr?limit=10&include_internal=true" \
      -H "Authorization: Bearer $TOKEN" | jq

    # Asterisk live state
    docker exec asterisk-lab asterisk -rx "pjsip show endpoints"
    docker exec asterisk-lab asterisk -rx "queue show"

    # Logs
    docker logs asterisk-lab --tail 50
    docker logs asterisk-pg --tail 50
    docker logs asterisk-pg-backup --tail 30

    # Container status
    docker compose ps

    # Reset everything (DESTRUCTIVE)
    docker compose down -v && docker compose up -d

---

## Appendix B — Common tasks cheat sheet

**Add a new extension**

    Admin → Extensions → New extension → fill in → Save

**Add a new queue**

    Admin → Queues → New queue → fill in → Save

**Add a new agent**

    Admin → Agents → New agent → fill in → Create
    → COPY the credentials from the modal.

**Reset an agent's password**

    Admin → Agents → Reset password on the agent's row
    → COPY the new password from the modal.

**Listen to a call recording**

    Admin → Calls → Details on the row → play in the audio player

**Free disk space from old recordings**

    Admin → Settings → Recordings retention → choose preset → Save
    → click "Run cleanup now"

**Change the recordings retention policy**

    Admin → Settings → Recordings retention → choose preset → Save

**Force Asterisk to reload the queue config**

    Admin → Queues → Reload Queues

**Force Asterisk to reload PJSIP**

    Admin → Extensions → Reload Asterisk

---

## Appendix C — Environment variables

`backend/.env`:

| Variable | Default | Notes |
|---|---|---|
| `AMI_HOST` | `127.0.0.1` | Asterisk AMI host |
| `AMI_PORT` | `5038` | Asterisk AMI port |
| `AMI_USER` | — | AMI username |
| `AMI_PASS` | — | AMI password |
| `WS_BIND` | `0.0.0.0:8080` | Where the backend listens |
| `RUST_LOG` | `info` | Log level. `debug` for troubleshooting |
| `DATABASE_URL` | — | `postgres://user:pass@host:port/db` |
| `JWT_SECRET` | — | Must be ≥32 chars |
| `ACCESS_TOKEN_TTL_MINUTES` | `15` | |
| `REFRESH_TOKEN_TTL_DAYS` | `7` | |
| `SEED_DEV_USERS` | `false` | Set `true` for dev to create `admin` and `agent1` |
| `APP_ENV` | `production` | `development` allows dev secrets |

**Production checklist:**

- `APP_ENV=production`
- `SEED_DEV_USERS=false`
- All secrets rotated (not the sample values)
- `RUST_LOG=warn` or `info`

---

## Appendix D — Emergency procedures

### Backend is hung

    Ctrl+C in the cargo run terminal
    cargo run

If it hangs on startup, check the migration status (§7).

### Postgres volume is corrupted

    # Restore from the most recent backup
    cd /home/thomas/Coding/Rust/call-me
    docker compose down
    docker compose up -d postgres
    sleep 10
    ./scripts/restore-db.sh asterisk-backups/$(ls -1t asterisk-backups/ | head -1)
    docker compose up -d
    cd backend && cargo run

### Asterisk is not responding to AMI

    docker restart asterisk-lab
    sleep 10
    docker logs asterisk-lab --tail 30

Then restart the backend so it re-establishes the AMI connection.

### You lost the admin password

    # Stop the backend
    # Update the password hash directly
    # (You need a new hash — easiest way is to temporarily enable
    #  SEED_DEV_USERS, create a fresh user, then disable it again.)

Or reset via SQL if you have a precomputed hash.

### Disk is full

    df -h

    # Check recordings size
    du -sh asterisk-recordings/

    # Check backups size
    du -sh asterisk-backups/

    # Reduce retention and run cleanup
    Admin → Settings → Retention → 30 days → Save → Run cleanup now

---

*Last updated: 2026-09-26*
