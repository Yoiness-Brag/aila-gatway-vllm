# Database Migration Guide

## Overview

This project uses SQLx for database migrations with compile-time query verification. Migrations are versioned using timestamps and stored in the `migrations/` folder.

## Prerequisites

- PostgreSQL 15+
- Docker and Docker Compose
- (Optional) sqlx-cli for local development

## Migration Versioning

Migrations use timestamp-based naming format:

```
YYYYMMDDHHMMSS_description.sql
```

Example: `20250601000000_create_oauth_services_table.sql`

SQLx tracks applied migrations in the `_sqlx_migrations` table.

## Current Migration Files

| File | Description |
|------|-------------|
| 20250515052350_initial_ee_schema.sql | Initial schema with providers table |
| 20250515125629_create_model_definitions_table.sql | Model definitions table |
| 20250518073526_create_pipelines_and_plugins_tables.sql | Pipelines and plugin configs |
| 20250601000000_create_oauth_services_table.sql | OAuth services and usage logs |

## Running Migrations

### Using Docker Compose (Recommended)

```bash
# Start PostgreSQL
docker compose up -d postgres

# Run migrations
docker compose up migrations
```

### Using sqlx-cli (Local Development)

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Set database URL
export DATABASE_URL=postgresql://hid_oauth:hidoauthpassword@localhost:5432/hid_oauth

# Run migrations
sqlx migrate run
```

## Adding New Tables or Columns

### Step 1: Create Migration File

```bash
# Create new migration with timestamp
# Format: migrations/YYYYMMDDHHMMSS_description.sql

# Example for adding a new column:
cat > migrations/20250620000000_add_custom_field.sql << 'EOF'
ALTER TABLE hub_llmgateway_oauth_clients
ADD COLUMN custom_field VARCHAR(255);
EOF

# Example for creating a new table:
cat > migrations/20250620000001_create_audit_logs.sql << 'EOF'
CREATE TABLE hub_llmgateway_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action VARCHAR(50) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    user_id UUID,
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_entity ON hub_llmgateway_audit_logs(entity_type, entity_id);
CREATE INDEX idx_audit_logs_created_at ON hub_llmgateway_audit_logs(created_at DESC);
EOF
```

### Step 2: Update Rust Code

1. Update model struct in `src/management/db/models.rs`
2. Update repository queries in `src/management/db/repositories/`
3. Update DTOs in `src/management/dto.rs` if needed

### Step 3: Run Migration

```bash
docker compose up migrations
```

### Step 4: Regenerate SQLx Cache

```bash
docker compose up sqlx-prepare
```

### Step 5: Rebuild Gateway

```bash
docker compose -f docker-compose-hid-oauth.yml build hid-oauth-gateway
```

## SQLx Offline Mode

The project uses `SQLX_OFFLINE=true` for Docker builds. This requires pre-generated `.sqlx` cache files.

### Why Cache Files Are Needed

SQLx verifies SQL queries at compile time. Without a database connection, it uses cached query metadata from `.sqlx/*.json` files.

### When to Regenerate Cache

Regenerate `.sqlx` cache when you:

- Add or remove columns
- Change column types
- Add new tables
- Modify SQL queries in repository files

### Regenerating Cache

```bash
# Using docker-compose service
docker compose up sqlx-prepare

# Or manually with sqlx-cli
export DATABASE_URL=postgresql://hid_oauth:hidoauthpassword@localhost:5432/hid_oauth
cargo sqlx prepare
```

## Database Schema

### hub_llmgateway_oauth_clients

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| application_service | VARCHAR(255) | Service name |
| application_id | UUID | Unique identifier |
| api_key_hash | VARCHAR(255) | Hashed API key |
| api_secret_hash | VARCHAR(255) | Hashed API secret |
| rate_limit_per_minute | INTEGER | Rate limit (default: 100) |
| allowed_models | JSONB | Allowed model list |
| metadata | JSONB | Custom metadata |
| is_active | BOOLEAN | Active status |
| token_expiration_hours | INTEGER | Token TTL (default: 24) |
| expires_at | TIMESTAMPTZ | Expiration timestamp |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update timestamp |
| last_used_at | TIMESTAMPTZ | Last usage timestamp |

### hub_llmgateway_client_usage_logs

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| oauth_service_id | UUID | FK to oauth_clients |
| endpoint | VARCHAR(255) | Called endpoint |
| model | VARCHAR(255) | Model used |
| tokens_used | INTEGER | Tokens consumed |
| latency_ms | INTEGER | Response latency |
| response_status | INTEGER | HTTP status code |
| error_message | TEXT | Error details |
| request_timestamp | TIMESTAMPTZ | Request time |

### hub_llmgateway_providers

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| name | VARCHAR(255) | Provider name (unique) |
| provider_type | VARCHAR(50) | Type (openai, bedrock, etc.) |
| base_url | VARCHAR(500) | API base URL |
| api_key_env_var | VARCHAR(100) | Environment variable for API key |
| enabled | BOOLEAN | Enabled status |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update timestamp |

### hub_llmgateway_model_definitions

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| key | VARCHAR(255) | Model key (unique) |
| provider_id | UUID | FK to providers |
| model_name | VARCHAR(255) | Actual model name |
| config | JSONB | Model configuration |
| enabled | BOOLEAN | Enabled status |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update timestamp |

### hub_llmgateway_pipelines

| Column | Type | Description |
|--------|------|-------------|
| id | UUID | Primary key |
| name | VARCHAR(255) | Pipeline name (unique) |
| route_path | VARCHAR(255) | API route path |
| model_key | VARCHAR(255) | Model key reference |
| enabled | BOOLEAN | Enabled status |
| created_at | TIMESTAMPTZ | Creation timestamp |
| updated_at | TIMESTAMPTZ | Last update timestamp |

## Migration Commands

```bash
# Run pending migrations
sqlx migrate run

# Check migration status
sqlx migrate info

# Revert last migration
sqlx migrate revert

# Create new migration file
sqlx migrate add <migration_name>
```

## Troubleshooting

### "no cached data for this query"

Cause: .sqlx cache files are outdated.

Solution:
```bash
docker compose up sqlx-prepare
docker compose -f docker-compose-hid-oauth.yml build hid-oauth-gateway
```

### "relation already exists"

Cause: Migration partially applied or table exists.

Solution:
```bash
# Check migration status
docker exec hid-oauth-postgres psql -U hid_oauth -d hid_oauth \
  -c "SELECT * FROM _sqlx_migrations;"

# If needed, drop and recreate (WARNING: destroys data)
docker exec hid-oauth-postgres psql -U hid_oauth -d hid_oauth \
  -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
docker compose up migrations
```

### "permission denied"

Cause: Database user lacks permissions.

Solution:
```sql
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO hid_oauth;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO hid_oauth;
```

### Connection refused

Cause: PostgreSQL not running or wrong connection string.

Solution:
```bash
# Check PostgreSQL status
docker compose ps

# Verify connection
docker exec hid-oauth-postgres psql -U hid_oauth -d hid_oauth -c "SELECT 1;"
```

## Docker Files

### Dockerfile.db.migrations

Runs migrations in a container:

```dockerfile
FROM rust:1.88-trixie AS builder
RUN cargo install sqlx-cli --version 0.8.6 --no-default-features --features postgres,native-tls --locked

FROM gcr.io/distroless/cc-debian13:debug-nonroot AS runtime
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY migrations /migrations
WORKDIR /
ENTRYPOINT ["sqlx", "migrate", "run"]
```

### Dockerfile.sqlx-prepare

Regenerates .sqlx cache files:

```dockerfile
FROM rust:1.88-trixie
WORKDIR /app
RUN cargo install sqlx-cli --version 0.8.6 --no-default-features --features postgres,native-tls --locked
COPY . .
CMD ["cargo", "sqlx", "prepare"]
```
