# Management API - Database Mode

Configure providers, models, and pipelines via REST API when running in Database Mode (`AILA_OAUTH_MODE=database`).

Configuration is stored in PostgreSQL and managed via API on port 8080. Changes are automatically picked up without restart (polling interval: 30s).

## Prerequisites

1. PostgreSQL database running
2. `AILA_OAUTH_MODE=database` environment variable set
3. `DATABASE_URL` configured
4. Migrations applied (`sqlx migrate run`)

## Provider Management

Providers are LLM backends (OpenAI, Anthropic, vLLM, etc.) that the gateway routes requests to.

### Create Provider

**POST** `/api/v1/management/providers`

```bash
curl -X POST http://localhost:8080/api/v1/management/providers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "vllm-aila",
    "provider_type": "OpenAI",
    "config": {
      "api_key": {"type": "literal", "value": "EMPTY"},
      "base_url": "http://vllm:8000/v1"
    },
    "enabled": true
  }'
```

**Provider Types:**

| Type | Description | Required Config |
|------|-------------|-----------------|
| `OpenAI` | OpenAI API (also vLLM) | `api_key`, optional `base_url`, `organization_id` |
| `Anthropic` | Anthropic Claude | `api_key` |
| `Azure` | Azure OpenAI | `api_key`, `resource_name`, `api_version`, optional `base_url` |
| `VertexAI` | Google VertexAI | `project_id`, `location`, optional `credentials_path`, `api_key` |
| `Bedrock` | AWS Bedrock | `region`, optional AWS credentials, `use_iam_role`, `inference_profile_id` |

**Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "aila-vllm",
  "provider_type": "openai",
  "config": {
    "api_key": {"type": "literal", "value": "***"},
    "base_url": "http://vllm-inference/v1"
  },
  "enabled": true,
  "created_at": "2025-06-01T00:00:00Z",
  "updated_at": "2025-06-01T00:00:00Z"
}
```

### List Providers

**GET** `/api/v1/management/providers`

```bash
curl http://localhost:8080/api/v1/management/providers
```

### Get Provider

**GET** `/api/v1/management/providers/{id}`

```bash
curl http://localhost:8080/api/v1/management/providers/550e8400-e29b-41d4-a716-446655440000
```

### Update Provider

**PUT** `/api/v1/management/providers/{id}`

```bash
curl -X PUT http://localhost:8080/api/v1/management/providers/550e8400-e29b-41d4-a716-446655440000 \
  -H "Content-Type: application/json" \
  -d '{
    "enabled": false
  }'
```

### Delete Provider

**DELETE** `/api/v1/management/providers/{id}`

```bash
curl -X DELETE http://localhost:8080/api/v1/management/providers/550e8400-e29b-41d4-a716-446655440000
```

## Model Definition Management

Models map user-facing model names to providers.

### Create Model

**POST** `/api/v1/management/model-definitions`

```bash
curl -X POST http://localhost:8080/api/v1/management/model-definitions \
  -H "Content-Type: application/json" \
  -d '{
    "key": "aila-ocr",
    "model_type": "allenai/olmOCR-2-7B-1025-FP8",
    "provider_id": "550e8400-e29b-41d4-a716-446655440000",
    "enabled": true
  }'
```

**Fields:**

| Field | Description |
|-------|-------------|
| `key` | User-facing model key (e.g., `aila-ocr`) |
| `model_type` | Actual model identifier at provider |
| `provider_id` | UUID of registered provider |
| `config_details` | Optional model-specific config (e.g., Azure deployment ID) |
| `enabled` | Whether model is active |

**Response:**

```json
{
  "id": "660e8400-e29b-41d4-a716-446655440001",
  "key": "aila-ocr",
  "model_type": "allenai/olmOCR-2-7B-1025-FP8",
  "provider_id": "550e8400-e29b-41d4-a716-446655440000",
  "enabled": true,
  "created_at": "2026-02-21T07:00:00Z"
}
```

### List Models

**GET** `/api/v1/management/model-definitions`

```bash
curl http://localhost:8080/api/v1/management/model-definitions
```

### Update Model

**PUT** `/api/v1/management/model-definitions/{id}`

```bash
curl -X PUT http://localhost:8080/api/v1/management/model-definitions/660e8400-e29b-41d4-a716-446655440001 \
  -H "Content-Type: application/json" \
  -d '{
    "enabled": false
  }'
```

### Delete Model

**DELETE** `/api/v1/management/model-definitions/{id}`

```bash
curl -X DELETE http://localhost:8080/api/v1/management/model-definitions/660e8400-e29b-41d4-a716-446655440001
```

## Pipeline Management

Pipelines define request processing workflows with plugins.

### Create Pipeline

**POST** `/api/v1/management/pipelines`

```bash
curl -X POST http://localhost:8080/api/v1/management/pipelines \
  -H "Content-Type: application/json" \
  -d '{
    "name": "default",
    "pipeline_type": "Chat",
    "plugins": [
      {
        "plugin_type": "logging",
        "config_data": {"level": "info"},
        "enabled": true,
        "order_in_pipeline": 0
      },
      {
        "plugin_type": "model-router",
        "config_data": {"models": [{"key": "aila-ocr", "priority": 0}]},
        "enabled": true,
        "order_in_pipeline": 1
      }
    ],
    "enabled": true
  }'
```

**Pipeline Types:**

| Type | Description |
|------|-------------|
| `Chat` | Chat completions (LLM and VLM) |
| `Completion` | Text completions |
| `Embeddings` | Text embeddings |

**Multi-Model Routing Example:**

```bash
curl -X POST http://localhost:8080/api/v1/management/pipelines \
  -H "Content-Type: application/json" \
  -d '{
    "name": "multi-model-pipeline",
    "pipeline_type": "Chat",
    "description": "Multi-model pipeline with fallback",
    "plugins": [{
      "plugin_type": "model-router",
      "config_data": {
        "models": [
          {"key": "aila-ocr", "priority": 0},
          {"key": "gpt-4-vision", "priority": 1},
          {"key": "claude-vision", "priority": 2}
        ]
      },
      "enabled": true,
      "order_in_pipeline": 0
    }],
    "enabled": true
  }'
```

Gateway tries models in priority order (0 = highest). Falls back to next model if previous fails.

**Plugin Types:**

| Plugin | Config | Description |
|--------|--------|-------------|
| `logging` | `{"level": "info"}` | Request/response logging |
| `model-router` | `{"models": [{"key": "model-name", "priority": 0}]}` | Route to specified models |
| `tracing` | `{"endpoint": "url", "public_key": {...}, "secret_key": {...}}` | OpenTelemetry tracing |

**Response:**

```json
{
  "id": "770e8400-e29b-41d4-a716-446655440002",
  "name": "default",
  "pipeline_type": "chat",
  "plugins": [...],
  "enabled": true,
  "created_at": "2025-06-01T00:00:00Z"
}
```

### List Pipelines

**GET** `/api/v1/management/pipelines`

```bash
curl http://localhost:8080/api/v1/management/pipelines
```

### Update Pipeline

**PUT** `/api/v1/management/pipelines/{id}`

```bash
curl -X PUT http://localhost:8080/api/v1/management/pipelines/770e8400-e29b-41d4-a716-446655440002 \
  -H "Content-Type: application/json" \
  -d '{
    "plugins": [
      {
        "plugin_type": "logging",
        "config_data": {"level": "debug"},
        "enabled": true,
        "order_in_pipeline": 0
      },
      {
        "plugin_type": "model-router",
        "config_data": {"models": [{"key": "aila-ocr", "priority": 0}, {"key": "gpt-4", "priority": 1}]},
        "enabled": true,
        "order_in_pipeline": 1
      }
    ]
  }'
```

### Delete Pipeline

**DELETE** `/api/v1/management/pipelines/{id}`

```bash
curl -X DELETE http://localhost:8080/api/v1/management/pipelines/770e8400-e29b-41d4-a716-446655440002
```

## Complete Setup Example

Here's a complete example of setting up the AILA OCR model:

```bash
# Step 1: Create the vLLM provider
curl -X POST http://localhost:8080/api/v1/management/providers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "aila-vllm",
    "provider_type": "openai",
    "config": {
      "api_key": {"type": "literal", "value": "YOUR_HF_TOKEN"},
      "base_url": "http://vllm-inference/v1"
    },
    "enabled": true
  }'

# Step 2: Create the OCR model definition
curl -X POST http://localhost:8080/api/v1/management/model-definitions \
  -H "Content-Type: application/json" \
  -d '{
    "model_name": "aila-ocr",
    "model_type": "allenai/olmOCR-2-7B-1025-FP8",
    "provider_name": "aila-vllm",
    "enabled": true
  }'

# Step 3: Create a pipeline with the model
curl -X POST http://localhost:8080/api/v1/management/pipelines \
  -H "Content-Type: application/json" \
  -d '{
    "name": "default",
    "pipeline_type": "Chat",
    "plugins": [
      {
        "plugin_type": "logging",
        "config_data": {"level": "info"},
        "enabled": true,
        "order_in_pipeline": 0
      },
      {
        "plugin_type": "model-router",
        "config_data": {"models": [{"key": "aila-ocr", "priority": 0}]},
        "enabled": true,
        "order_in_pipeline": 1
      }
    ],
    "enabled": true
  }'

# Step 4: Test the model (requires OAuth if REQUIRE_AUTH=true)
curl -X POST http://localhost:3000/api/v1/chat/completions \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "aila-ocr",
    "messages": [{"role": "user", "content": "Hello"}],
    "max_tokens": 100
  }'
```

## Equivalent config.yaml

The above API setup is equivalent to this `config.yaml` in YAML mode:

```yaml
providers:
  - key: aila-vllm
    type: openai
    api_key: YOUR_HF_TOKEN
    base_url: "http://vllm-inference/v1"

models:
  - key: aila-ocr
    type: allenai/olmOCR-2-7B-1025-FP8
    provider: aila-vllm

pipelines:
  - name: default
    type: Chat
    plugins:
      - Logging:
          level: info

## Configuration Polling

Changes are automatically picked up every 30 seconds (`DB_POLL_INTERVAL_SECONDS`).

## Error Responses

| Status | Error | Description |
|--------|-------|-------------|
| 400 | `invalid_request` | Invalid request body |
| 404 | `not_found` | Resource not found |
| 409 | `conflict` | Resource already exists |
| 500 | `server_error` | Internal server error |

**Example Error:**

```json
{
  "error": {
    "message": "Provider with name 'aila-vllm' already exists",
    "type": "conflict_error",
    "code": "conflict"
  }
}
```

## Database Schema

The Management API uses these PostgreSQL tables:

| Table | Purpose |
|-------|---------|
| `hub_llmgateway_providers` | LLM provider configurations |
| `hub_llmgateway_model_definitions` | Model to provider mappings |
| `hub_llmgateway_pipelines` | Pipeline configurations |
| `hub_llmgateway_pipeline_plugin_configs` | Plugin configurations |
| `hub_llmgateway_oauth_clients` | OAuth service credentials (application_service, application_id as UUID, token_expiration_hours) |
| `hub_llmgateway_client_usage_logs` | Request usage logs (oauth_service_id FK) |
