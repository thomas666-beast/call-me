#!/bin/sh
# ============================================================================
# backup-loop.sh — periodic Postgres backup for the callcenter database.
# ============================================================================
#
# Runs inside a postgres:16-alpine sidecar container. On startup it waits
# 20 seconds for Postgres to be fully ready, then takes a backup. After
# that it sleeps for BACKUP_INTERVAL_HOURS and repeats.
#
# Environment variables (set in docker-compose.yml):
#   PGHOST, PGPORT, PGDATABASE, PGUSER, PGPASSWORD  — connection
#   BACKUP_INTERVAL_HOURS  — how often to run (default 24)
#   BACKUP_RETENTION_DAYS  — delete backups older than this (default 30)
#   BACKUP_DIR             — where to write (default /backups)
#
# Backup files are named:
#   callcenter-YYYYMMDD-HHMMSS.sql.gz
# ============================================================================

set -eu

INTERVAL_HOURS="${BACKUP_INTERVAL_HOURS:-24}"
RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-30}"
BACKUP_DIR="${BACKUP_DIR:-/backups}"

mkdir -p "${BACKUP_DIR}"

echo "[backup] starting in 20s: interval=${INTERVAL_HOURS}h retention=${RETENTION_DAYS}d dir=${BACKUP_DIR}"
sleep 20

while true; do
    timestamp=$(date -u +%Y%m%d-%H%M%S)
    filename="callcenter-${timestamp}.sql.gz"
    tmp="${BACKUP_DIR}/${filename}.tmp"
    final="${BACKUP_DIR}/${filename}"

    echo "[backup] $(date -u +%Y-%m-%dT%H:%M:%SZ) dumping → ${filename}"

    if pg_dump --no-owner --no-privileges | gzip > "${tmp}"; then
        mv "${tmp}" "${final}"
        size=$(du -h "${final}" | cut -f1)
        echo "[backup] OK: ${filename} (${size})"
    else
        echo "[backup] FAILED — removing partial file"
        rm -f "${tmp}"
    fi

    echo "[backup] pruning files older than ${RETENTION_DAYS} days"
    find "${BACKUP_DIR}" -name 'callcenter-*.sql.gz' -type f -mtime +"${RETENTION_DAYS}" -print -delete || true

    echo "[backup] sleeping ${INTERVAL_HOURS}h"
    sleep $((INTERVAL_HOURS * 3600))
done
