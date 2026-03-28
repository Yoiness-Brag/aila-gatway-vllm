## Project Structure

```
hid-oauth/
├── src/                        # Main application code
│   ├── main.rs                 # Application entry point
│   ├── lib.rs                  # Library exports
│   ├── config/                 # Configuration management
│   ├── providers/              # LLM provider implementations
│   ├── models/                 # Data models
│   ├── pipelines/              # Request processing pipelines
│   ├── routes.rs               # HTTP routing
│   ├── state.rs                # Application state management
│   ├── management/             # Management API (Database mode)
│   │   ├── api/                # REST API endpoints
│   │   ├── db/                 # Database models and repositories
│   │   ├── services/           # Business logic
│   │   └── dto.rs              # Data transfer objects
│   └── types/                  # Shared type definitions
├── migrations/                 # Database migrations
├── helm/                       # Kubernetes deployment
├── tests/                      # Integration tests
└── docs/                       # Documentation
```

## Configuration Modes

### YAML Mode

Perfect for simple deployments and development environments.

**Features:**

- Static configuration via `config.yaml`
- No external dependencies
- Simple provider and model setup
- No management API
- Single port (3000)

**Example config.yaml:**

```yaml
providers:
  - key: openai
    type: openai
    api_key: sk-...

models:
  - key: gpt-4
    type: gpt-4
    provider: openai

pipelines:
  - name: chat
    type: Chat
    plugins:
      - ModelRouter:
          models: [gpt-4]
```

### Database Mode

Ideal for production environments requiring dynamic configuration.

**Features:**

- PostgreSQL-backed configuration
- REST Management API (`/api/v1/management/*`)
- Hot reload without restarts
- Configuration polling and synchronization
- SecretObject system for credential management
- Dual ports (3000 for Gateway, 8080 for Management)

**Setup:**

1. Set up PostgreSQL database
2. Run migrations: `sqlx migrate run`
3. Set environment variables:

   ```bash
   HID_OAUTH_MODE=database
   DATABASE_URL=postgresql://user:pass@host:5432/db
   ```

## API Endpoints

### Core LLM Gateway (Both Modes)

**Port 3000:**

- `POST /api/v1/chat/completions` - Chat completions
- `POST /api/v1/completions` - Text completions  
- `POST /api/v1/embeddings` - Text embeddings
- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics
- `GET /swagger-ui` - OpenAPI documentation

### Management API (Database Mode Only)

**Port 8080:**

- `GET /health` - Management API health check
- `GET|POST|PUT|DELETE /api/v1/management/providers` - Provider management
- `GET|POST|PUT|DELETE /api/v1/management/model-definitions` - Model management
- `GET|POST|PUT|DELETE /api/v1/management/pipelines` - Pipeline management
- `GET|POST|PUT|DELETE /api/v1/management/oauth-services` - OAuth client management
- `POST /api/v1/management/oauth-services/{id}/rotate-key` - Rotate API key
- `POST /api/v1/management/oauth-services/{id}/regenerate-token` - Regenerate expired token
- `GET /api/v1/management/oauth-services/lookup` - Lookup client by name/email

## Provider Configuration

### OpenAI

```yaml
providers:
  - key: openai
    type: openai
    api_key: sk-...
    # Optional
    organization_id: org-...
    base_url: https://api.openai.com/v1
```

### Anthropic

```yaml
providers:
  - key: anthropic
    type: anthropic
    api_key: sk-ant-...
```

### Azure OpenAI

```yaml
providers:
  - key: azure
    type: azure
    api_key: your-key
    resource_name: your-resource
    api_version: "2023-05-15"
```

### AWS Bedrock

```yaml
providers:
  - key: bedrock
    type: bedrock
    region: us-east-1
    # Uses IAM roles or AWS credentials
```

### Google VertexAI

Supports two authentication modes that route to different Google APIs:

```yaml
# Option 1: API Key (uses Gemini Developer API)
providers:
  - key: vertexai
    type: vertexai
    api_key: your-gemini-api-key

# Option 2: Service Account (uses Vertex AI)
providers:
  - key: vertexai
    type: vertexai
    project_id: your-project
    location: us-central1
    credentials_path: /path/to/service-account.json
```

| Auth Method | API Endpoint | Use Case |
|-------------|--------------|----------|
| API Key | `generativelanguage.googleapis.com` | Simple setup, development |
| Service Account | `{location}-aiplatform.googleapis.com` | Enterprise, GCP-integrated |

### vLLM / OpenAI-Compatible (Self-Hosted)

For self-hosted models using vLLM or other OpenAI-compatible inference servers:

```bash
# Register via Management API (Database Mode)
curl -X POST http://localhost:8080/api/v1/management/providers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "aila-vllm",
    "provider_type": "openai",
    "config": {
      "api_key": {"type": "literal", "value": "your-api-key"},
      "base_url": "http://your-vllm-host/v1"
    }
  }'
```

See [docs/VLLM_OCR_SETUP.md](docs/VLLM_OCR_SETUP.md) for detailed vLLM setup instructions.

## OAuth Client Authentication

Hid-OAuth supports OAuth client authentication for incoming requests (Database Mode only).

### Features

- **Configurable Token Expiration**: API keys expire based on `token_expiration_hours` (default: 24h)
- **Token Regeneration**: Regenerate expired tokens via API
- **Application ID**: Auto-generated unique identifier for each application
- **Client Lookup**: Find clients by application_service or application_id
- **Rate Limiting**: Per-client configurable rate limits
- **Usage Logging**: Track all API requests per client

### Quick Start

```bash
# Create OAuth client
curl -X POST http://localhost:8080/api/v1/management/oauth-services \
  -H "Content-Type: application/json" \
  -d '{"application_service": "My-OCR-App", "token_expiration_hours": 24}'

# Use returned API key for gateway requests
curl -X POST http://localhost:3000/api/v1/chat/completions \
  -H "Authorization: Bearer hid_live_xxx" \
  -H "Content-Type: application/json" \
  -d '{"model": "gpt-4", "messages": [{"role": "user", "content": "Hello"}]}'

# Regenerate expired token
curl -X POST http://localhost:8080/api/v1/management/oauth-services/{id}/regenerate-token
```

See [docs/OAUTH_IMPLEMENTATION.md](docs/OAUTH_IMPLEMENTATION.md) for full documentation.

## Deployment

### Helm Chart

```bash
# YAML Mode
helm install hid-oauth ./helm

# Database Mode
helm install hid-oauth ./helm \
  --set management.enabled=true \
  --set management.database.host=postgres \
  --set management.database.existingSecret=postgres-secret
```

### Docker Compose

[docker compose example](./example/docker/README.md)

```yaml
version: '3.8'
services:
  # Database Mode with OAuth
  hid-oauth-gateway:
    image: ocp-registry.aila.cloud/youness_elbrag/hid-oauth
    ports:
      - "3000:3000"
      - "8080:8080"
    environment:
      - HID_OAUTH_MODE=database
      - DATABASE_URL=postgresql://hid_oauth:password@postgres:5432/hid_oauth
      - REQUIRE_AUTH=true
    depends_on:
      - postgres

  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=hid_oauth
      - POSTGRES_USER=hid_oauth
      - POSTGRES_PASSWORD=password
```

## Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `HID_OAUTH_MODE` | Deployment mode: `yaml` or `database` | `yaml` | No |
| `CONFIG_FILE_PATH` | Path to YAML config file | `config.yaml` | YAML mode |
| `DATABASE_URL` | PostgreSQL connection string | - | Database mode |
| `DB_POLL_INTERVAL_SECONDS` | Config polling interval | `30` | No |
| `PORT` | Gateway server port | `3000` | No |
| `MANAGEMENT_PORT` | Management API port | `8080` | Database mode |
| `REQUIRE_AUTH` | Enable OAuth authentication on gateway | `false` | No |
| `TRACE_CONTENT_ENABLED` | Enable request/response tracing | `true` | No |

## Development

### Prerequisites

- Rust 1.87+
- PostgreSQL 12+ (for database mode)
- `sqlx-cli` (for migrations)

### Commands

```bash
# Build OSS version
cargo build

# Test
cargo test

# Format
cargo fmt

# Lint
cargo clippy

# Run YAML mode
cargo run

# Run database mode
HID_OAUTH_MODE=database DATABASE_URL=postgresql://... cargo run
```

### Database Setup (for Database Mode)

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run

# Use setup script for complete setup
./scripts/setup-db.sh
```

### Project Structure

The project follows a unified single-crate architecture:

- **`src/main.rs`**: Application entry point with mode detection
- **`src/lib.rs`**: Library exports for all modules
- **`src/config/`**: Configuration management and validation
- **`src/providers/`**: LLM provider implementations
- **`src/models/`**: Request/response data models
- **`src/pipelines/`**: Request processing pipelines
- **`src/management/`**: Management API (Database mode)
- **`src/types/`**: Shared type definitions
- **`src/state.rs`**: Thread-safe application state
- **`src/routes.rs`**: Dynamic HTTP routing

### Key Features

- **Hot Reload**: Configuration changes without restarts (Database mode)
- **Atomic Updates**: Thread-safe configuration updates
- **Dynamic Routing**: Pipeline-based request steering
- **Comprehensive Testing**: Integration tests with testcontainers
- **OpenAPI Documentation**: Auto-generated API specs

## Observability

### OpenTelemetry Tracing

Configure in your pipeline:

```yaml
pipelines:
  - name: traced-chat
    type: Chat
    plugins:
      - Tracing:
          endpoint: http://jaeger:14268/api/traces
          api_key: your-key
      - ModelRouter:
          models: [gpt-4]
```

### Prometheus Metrics

Available at `/metrics`:

- Request counts and latencies
- Provider-specific metrics
- Error rates
- Active connections

## Architecture

```mermaid
graph TB
    subgraph "Client Layer"
        C[Client Application]
    end

    subgraph "Hid-OAuth Gateway"
        AUTH[Auth Middleware]
        GW[Gateway :3000]
        MGMT[Management API :8080]
        PIPE[Pipeline Router]
    end

    subgraph "Data Layer"
        PG[(PostgreSQL)]
    end

    subgraph "LLM Providers"
        OPENAI[OpenAI]
        ANTHROPIC[Anthropic]
        AZURE[Azure OpenAI]
        VLLM[vLLM/AILA OCR]
    end

    C -->|Bearer Token| AUTH
    AUTH --> GW
    GW --> PIPE
    PIPE --> OPENAI
    PIPE --> ANTHROPIC
    PIPE --> AZURE
    PIPE --> VLLM
    MGMT --> PG
    AUTH --> PG
```

### Request Flow

```mermaid
sequenceDiagram
    participant Client
    participant Auth
    participant Gateway
    participant Provider

    Client->>Auth: POST /api/v1/chat/completions
    Auth->>Auth: Validate Bearer Token
    Auth->>Auth: Check Token Expiration
    Auth->>Auth: Check Rate Limit
    Auth->>Gateway: Authenticated Request
    Gateway->>Provider: Forward to LLM
    Provider-->>Gateway: Response
    Gateway-->>Client: JSON Response
```

### Docker Compose Deployment

```bash
docker-compose up -d
```

Services:
- **postgres**: PostgreSQL database (port 5432)
- **migrations**: Runs database migrations automatically
- **hid-oauth-gateway**: Gateway with OAuth authentication (ports 3000, 8080)
