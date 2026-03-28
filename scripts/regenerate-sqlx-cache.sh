#!/bin/bash
# Script to regenerate .sqlx offline cache files
# This must be run with a live database connection

set -e

echo "=== Regenerating SQLX Offline Cache ==="

# Check if DATABASE_URL is set
if [ -z "$DATABASE_URL" ]; then
    echo "Error: DATABASE_URL environment variable is not set"
    echo "Example: export DATABASE_URL=postgresql://aila_oauth:ailaoauthpassword@localhost:5432/aila_oauth"
    exit 1
fi

# Run migrations first
echo "1. Running database migrations..."
sqlx migrate run

# Clean old cache files
echo "2. Cleaning old .sqlx cache files..."
rm -rf .sqlx/*.json

# Regenerate cache
echo "3. Regenerating .sqlx cache files..."
cargo sqlx prepare

echo "=== Done! .sqlx cache files have been regenerated ==="
echo "You can now build with SQLX_OFFLINE=true"
