.PHONY: help postgres-up postgres-down migrate sqlx-prepare build-gateway up-gateway down-gateway logs clean-all

help:
	@echo "AILA OAuth Gateway - Docker Compose Commands"
	@echo ""
	@echo "Database Commands:"
	@echo "  make postgres-up        - Start PostgreSQL"
	@echo "  make postgres-down      - Stop PostgreSQL"
	@echo "  make migrate            - Run database migrations"
	@echo "  make sqlx-prepare       - Regenerate .sqlx cache files"
	@echo ""
	@echo "Gateway Commands:"
	@echo "  make build-gateway      - Build gateway Docker image"
	@echo "  make up-gateway         - Start gateway service"
	@echo "  make down-gateway       - Stop gateway service"
	@echo "  make logs               - View gateway logs"
	@echo ""
	@echo "Full Stack Commands:"
	@echo "  make up                 - Start PostgreSQL, run migrations, start gateway"
	@echo "  make down               - Stop all services"
	@echo "  make rebuild            - Rebuild and restart gateway"
	@echo "  make clean-all          - Stop and remove all containers"
	@echo ""
	@echo "OAuth Service Commands:"
	@echo "  make create-oauth       - Create OAuth service"
	@echo "  make list-oauth         - List OAuth services"
	@echo "  make test-auth          - Test authenticated request"
	@echo ""
	@echo "Management API Commands:"
	@echo "  make create-provider    - Create LLM provider"
	@echo "  make list-providers     - List providers"
	@echo "  make create-model       - Create model definition"
	@echo "  make list-models        - List models"
	@echo "  make create-pipeline    - Create pipeline"
	@echo "  make list-pipelines     - List pipelines"
	@echo ""
	@echo "Health Check Commands:"
	@echo "  make health-gateway     - Check gateway health (port 3000)"
	@echo "  make health-mgmt        - Check management API health (port 8080)"
	@echo "  make metrics            - View Prometheus metrics"

GATEWAY_PORT := 3000
MANAGEMENT_PORT := 8080

postgres-up:
	@echo "Starting PostgreSQL..."
	docker compose up -d postgres

postgres-down:
	@echo "Stopping PostgreSQL..."
	docker compose down postgres

migrate:
	@echo "Running database migrations..."
	docker compose up migrations

sqlx-prepare:
	@echo "Regenerating .sqlx cache files..."
	docker compose up sqlx-prepare

build-gateway:
	@echo "Building gateway Docker image..."
	docker compose -f docker-compose-ailaoauth.yml build aila-oauth-gateway

up-gateway:
	@echo "Starting gateway service..."
	docker compose -f docker-compose-ailaoauth.yml up -d aila-oauth-gateway
	@echo "Gateway running on:"
	@echo "  LLM Gateway:    http://localhost:$(GATEWAY_PORT)"
	@echo "  Management API: http://localhost:$(MANAGEMENT_PORT)"
	@echo "  Swagger UI:     http://localhost:$(MANAGEMENT_PORT)/swagger-ui/"

down-gateway:
	@echo "Stopping gateway service..."
	docker compose -f docker-compose-ailaoauth.yml down

up: postgres-up migrate up-gateway
	@echo ""
	@echo "Full stack started successfully!"

down:
	@echo "Stopping all services..."
	docker compose -f docker-compose-ailaoauth.yml down
	docker compose down

rebuild: down-gateway sqlx-prepare build-gateway up-gateway
	@echo "Gateway rebuilt and restarted!"

clean-all:
	@echo "Stopping and removing all containers..."
	docker stop $$(docker ps -aq) 2>/dev/null || true
	docker rm $$(docker ps -aq) 2>/dev/null || true
	@echo "Cleanup complete!"

logs:
	@echo "Viewing gateway logs..."
	docker logs -f aila-oauth-gateway

create-oauth:
	@echo "Creating OAuth service..."
	@echo "Usage: make create-oauth SERVICE=<service_name>"
	@if [ -z "$(SERVICE)" ]; then \
		echo "Error: SERVICE is required"; \
		echo "Example: make create-oauth SERVICE='OCR-Invoice-Service'"; \
		exit 1; \
	fi
	curl -X POST http://localhost:$(MANAGEMENT_PORT)/api/v1/management/oauth-services \
		-H "Content-Type: application/json" \
		-d '{"application_service": "$(SERVICE)", "rate_limit_per_minute": 100, "allowed_models": ["aila-ocr"], "token_expiration_hours": 24}' | jq .

list-oauth:
	@echo "Listing OAuth services..."
	curl -s http://localhost:$(MANAGEMENT_PORT)/api/v1/management/oauth-services | jq .

rotate-key:
	@echo "Rotating API key..."
	@if [ -z "$(ID)" ]; then \
		echo "Error: ID is required"; \
		echo "Example: make rotate-key ID='<uuid>'"; \
		exit 1; \
	fi
	curl -X POST "http://localhost:$(MANAGEMENT_PORT)/api/v1/management/oauth-services/$(ID)/rotate-key" | jq .

test-auth:
	@echo "Testing authenticated request..."
	@if [ -z "$(TOKEN)" ]; then \
		echo "Error: TOKEN is required"; \
		echo "Example: make test-auth TOKEN='aila_live_xxx'"; \
		exit 1; \
	fi
	curl -X POST http://localhost:$(GATEWAY_PORT)/api/v1/chat/completions \
		-H "Authorization: Bearer $(TOKEN)" \
		-H "Content-Type: application/json" \
		-d '{"model": "aila-ocr", "messages": [{"role": "user", "content": "Hello"}]}'

create-provider:
	@echo "Creating LLM provider..."
	@echo "Usage: make create-provider NAME=<name> TYPE=<type> URL=<url> KEY=<key>"
	@if [ -z "$(NAME)" ] || [ -z "$(TYPE)" ]; then \
		echo "Error: NAME and TYPE are required"; \
		echo "Example: make create-provider NAME='openai' TYPE='openai' URL='https://api.openai.com/v1' KEY='sk-xxx'"; \
		exit 1; \
	fi
	curl -X POST http://localhost:$(MANAGEMENT_PORT)/api/v1/management/providers \
		-H "Content-Type: application/json" \
		-d '{"name": "$(NAME)", "provider_type": "$(TYPE)", "config": {"api_key": {"type": "literal", "value": "$(KEY)"}, "base_url": "$(URL)"}, "enabled": true}' | jq .

list-providers:
	@echo "Listing providers..."
	curl -s http://localhost:$(MANAGEMENT_PORT)/api/v1/management/providers | jq .

create-model:
	@echo "Creating model definition..."
	@echo "Usage: make create-model KEY=<key> PROVIDER=<provider> MODEL=<model>"
	@if [ -z "$(KEY)" ] || [ -z "$(PROVIDER)" ] || [ -z "$(MODEL)" ]; then \
		echo "Error: KEY, PROVIDER, and MODEL are required"; \
		echo "Example: make create-model KEY='gpt-4' PROVIDER='openai' MODEL='gpt-4'"; \
		exit 1; \
	fi
	curl -X POST http://localhost:$(MANAGEMENT_PORT)/api/v1/management/model-definitions \
		-H "Content-Type: application/json" \
		-d '{"key": "$(KEY)", "provider_id": "$(PROVIDER)", "model_name": "$(MODEL)", "enabled": true}' | jq .

list-models:
	@echo "Listing model definitions..."
	curl -s http://localhost:$(MANAGEMENT_PORT)/api/v1/management/model-definitions | jq .

create-pipeline:
	@echo "Creating pipeline..."
	@echo "Usage: make create-pipeline NAME=<name> ROUTE=<route> MODEL=<model>"
	@if [ -z "$(NAME)" ] || [ -z "$(ROUTE)" ] || [ -z "$(MODEL)" ]; then \
		echo "Error: NAME, ROUTE, and MODEL are required"; \
		echo "Example: make create-pipeline NAME='default' ROUTE='/v1/chat/completions' MODEL='gpt-4'"; \
		exit 1; \
	fi
	curl -X POST http://localhost:$(MANAGEMENT_PORT)/api/v1/management/pipelines \
		-H "Content-Type: application/json" \
		-d '{"name": "$(NAME)", "route_path": "$(ROUTE)", "model_key": "$(MODEL)", "enabled": true}' | jq .

list-pipelines:
	@echo "Listing pipelines..."
	curl -s http://localhost:$(MANAGEMENT_PORT)/api/v1/management/pipelines | jq .

test-chat:
	@echo "Testing chat completions..."
	@echo "Usage: make test-chat MODEL=<model>"
	@if [ -z "$(MODEL)" ]; then \
		echo "Error: MODEL is required"; \
		echo "Example: make test-chat MODEL='gpt-4'"; \
		exit 1; \
	fi
	curl -X POST http://localhost:$(GATEWAY_PORT)/api/v1/chat/completions \
		-H "Content-Type: application/json" \
		-d '{"model": "$(MODEL)", "messages": [{"role": "user", "content": "Hello"}]}' | jq .

health-gateway:
	@echo "Checking LLM Gateway health (port 3000)..."
	curl -s http://localhost:$(GATEWAY_PORT)/health

health-mgmt:
	@echo "Checking Management API health (port 8080)..."
	curl -s http://localhost:$(MANAGEMENT_PORT)/health

metrics:
	@echo "Viewing Prometheus metrics (port 3000)..."
	curl -s http://localhost:$(GATEWAY_PORT)/metrics
