#!/usr/bin/env bash
# Generate fresh secrets for the Asterisk Manager.
# Copy the output into backend/.env and docker-compose.yml.

set -euo pipefail

echo "# ----------------------------------------------------------------------------"
echo "# Fresh secrets — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "# ----------------------------------------------------------------------------"
echo
echo "JWT_SECRET=$(openssl rand -hex 32)"
echo "PGPASSWORD=$(openssl rand -hex 16)"
echo "AMI_PASSWORD=$(openssl rand -hex 16)"
echo
echo "# ----------------------------------------------------------------------------"
echo "# Apply these to:"
echo "#   backend/.env      -> JWT_SECRET, AMI_PASS, DATABASE_URL password"
echo "#   docker-compose.yml -> POSTGRES_PASSWORD (for fresh installs only)"
echo "#   asterisk-config/manager.conf -> secret = <AMI_PASSWORD>"
echo "#"
echo "# Then:"
echo "#   docker exec asterisk-lab asterisk -rx 'manager reload'"
echo "#   docker exec -it asterisk-pg psql -U callcenter -d callcenter \\"
echo "#     -c \"ALTER USER callcenter WITH PASSWORD '<PGPASSWORD>';\""
echo "# ----------------------------------------------------------------------------"
