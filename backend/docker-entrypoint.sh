#!/bin/bash
set -e

echo "🔄 Running database migrations..."

# Run each migration file in order
for migration in /app/migrations/*.sql; do
    if [ -f "$migration" ]; then
        echo "Running: $(basename $migration)"
        PGPASSWORD="${POSTGRES_PASSWORD:-coreauth_dev_password}" psql \
            -h "${POSTGRES_HOST:-postgres}" \
            -p "${POSTGRES_PORT:-5432}" \
            -U "${POSTGRES_USER:-coreauth}" \
            -d "${POSTGRES_DB:-coreauth}" \
            -f "$migration" 2>&1 | grep -v "NOTICE: relation" || true
    fi
done

echo "✅ Migrations complete"
echo ""
echo "🚀 Starting CoreAuth API..."

# Start the application
exec ./coreauth-api
