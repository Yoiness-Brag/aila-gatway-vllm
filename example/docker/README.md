# AILA-OAuth Docker Compose Example

## Quick Start

```bash
cp env.template .env
docker compose up -d
```

## Services

- **aila-oauth-gateway**: LLM Gateway with OAuth (ports 3000, 8080)
- **aila-oauth-migrations**: Database migrations
- **postgres**: PostgreSQL database (port 5432)

## Commands

```bash
docker compose up -d
docker compose down
docker compose logs -f aila-oauth-gateway
```

## Test

```bash
curl http://localhost:3000/health
curl http://localhost:8080/health

curl -X POST http://localhost:8080/api/v1/management/oauth-services \
  -H "Content-Type: application/json" \
  -d '{"application_service": "Test-App", "token_expiration_hours": 24}'
```
