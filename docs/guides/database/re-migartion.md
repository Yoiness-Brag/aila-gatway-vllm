# AILA OAuth Gateway - Docker and Migration Guide

## Docker Compose Files

| File | Purpose |
|------|---------|
| `docker-compose.yml` | PostgreSQL + Migrations + sqlx-prepare |
| `docker-compose-aila-oauth.yml` | Gateway service |

## Quick Start

```bash
docker compose up -d postgres
docker compose up migrations
docker compose -f docker-compose-aila-oauth.yml up -d
```

## Migration Files

Migrations use timestamp naming: `YYYYMMDDHHMMSS_description.sql`

Current migrations:
- `20250515052350_initial_ee_schema.sql` - Providers
- `20250515125629_create_model_definitions_table.sql` - Models
- `20250518073526_create_pipelines_and_plugins_tables.sql` - Pipelines
- `20250601000000_create_oauth_services_table.sql` - OAuth services

## Adding Schema Changes

```bash
# 1. Create migration file (use timestamp format)
cat > migrations/20250620000000_add_feature.sql << 'EOF'
ALTER TABLE hub_llmgateway_oauth_clients ADD COLUMN new_field VARCHAR(255);
EOF

# 2. Run migration
docker compose up migrations

# 3. Update Rust models in src/management/db/models.rs

# 4. Regenerate SQLx cache
docker compose up sqlx-prepare

# 5. Rebuild gateway
docker compose -f docker-compose-aila-oauth.yml build
```

## SQLx Cache

SQLx uses compile-time verification. Regenerate cache after schema changes:

```bash
docker compose up sqlx-prepare
```


## Schema Tables

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

## Troubleshooting

**Build fails with "no cached data":**
```bash
docker compose up sqlx-prepare
docker compose -f docker-compose-aila-oauth.yml build
```

**Migration fails with "relation already exists":**
```bash
docker exec aila-oauth-postgres psql -U aila_oauth -d aila_oauth \
  -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"
docker compose up migrations
```
