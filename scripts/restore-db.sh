#!/usr/bin/env bash
# ============================================================================
# restore-db.sh — restore a backup into the running callcenter database.
# ============================================================================
#
# WARNING: This DROPS and RECREATES the entire schema.
# Stop the backend first to avoid partial writes.
#
# Usage:
#   ./scripts/restore-db.sh asterisk-backups/callcenter-20260926-120000.sql.gz
#
# If no argument is given, lists the most recent backups.
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BACKUP_DIR="${SCRIPT_DIR}/../asterisk-backups"

if [ $# -ne 1 ]; then
    echo "Usage: $0 <backup-file.sql.gz>"
    echo ""
    echo "Most recent backups:"
    if ls -1t "${BACKUP_DIR}"/callcenter-*.sql.gz >/dev/null 2>&1; then
        ls -1th "${BACKUP_DIR}"/callcenter-*.sql.gz | head -10
    else
        echo "  (none found in ${BACKUP_DIR})"
    fi
    exit 1
fi

BACKUP_FILE="$1"
if [ ! -f "$BACKUP_FILE" ]; then
    echo "Backup file not found: $BACKUP_FILE"
    exit 1
fi

PGPASSWORD="${PGPASSWORD:-callcenter_dev_password}"

echo "This will DROP and RECREATE all data in the callcenter database."
echo "Backup: $BACKUP_FILE"
echo ""
read -p "Type 'yes' to continue: " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
    echo "Aborted."
    exit 1
fi

echo ""
echo "[1/2] Dropping and recreating public schema..."
docker exec -e PGPASSWORD="$PGPASSWORD" -i asterisk-pg \
    psql -U callcenter -d callcenter -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"

echo "[2/2] Restoring from backup..."
gunzip -c "$BACKUP_FILE" | docker exec -e PGPASSWORD="$PGPASSWORD" -i asterisk-pg \
    psql -U callcenter -d callcenter

echo ""
echo "Restore complete."
echo ""
echo "Next steps:"
echo "  1. Restart the backend:  cd backend && cargo run"
echo "  2. Log in and verify the data is intact."
