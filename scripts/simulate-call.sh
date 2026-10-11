#!/usr/bin/env bash
# ============================================================================
# simulate-call.sh — originate a fake inbound call for testing.
# ============================================================================
#
# Usage:
#   ./simulate-call.sh                      # random fake number, queue 2000
#   ./simulate-call.sh +15551000007         # specific number, queue 2000
#   ./simulate-call.sh +15551000007 2001    # specific number, specific queue
# ============================================================================

set -euo pipefail

AMI_HOST="${AMI_HOST:-127.0.0.1}"
AMI_PORT="${AMI_PORT:-5038}"
AMI_USER="${AMI_USER:-rustapp}"
AMI_PASS="${AMI_PASS:-c04b9d184c0207a076db2b92851c2a0e}"

NUMBER="${1:-}"
QUEUE="${2:-2001}"

if [ -z "$NUMBER" ]; then
    N=$(( (RANDOM % 20) + 1 ))
    NUMBER=$(printf "+155510000%02d" "$N")
fi

echo "Simulating call from $NUMBER to queue $QUEUE"

{
    printf 'Action: Login\r\n'
    printf 'Username: %s\r\n' "$AMI_USER"
    printf 'Secret: %s\r\n' "$AMI_PASS"
    printf 'Events: off\r\n'
    printf '\r\n'
    sleep 0.3

    printf 'Action: Originate\r\n'
    printf 'Channel: Local/%s@from-internal\r\n' "$QUEUE"
    printf 'Application: Wait\r\n'
    printf 'Data: 60\r\n'
    printf 'CallerID: "Test Caller" <%s>\r\n' "$NUMBER"
    printf 'Async: true\r\n'
    printf 'ActionID: sim-%s\r\n' "$(date +%s)"
    printf '\r\n'
    sleep 0.5

    printf 'Action: Logoff\r\n'
    printf '\r\n'
    sleep 0.2
} | nc "$AMI_HOST" "$AMI_PORT" > /dev/null 2>&1 || true

echo "Originated (caller will wait up to 60s in the queue)."
echo "Watch the queue: docker exec asterisk-lab asterisk -rx \"queue show $QUEUE\""
