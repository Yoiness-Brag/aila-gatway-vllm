# AILA-OAuth API Reference

## Gateway API (Port 3000)

### Chat Completions

**POST** `/api/v1/chat/completions`

**Text Request:**
```json
{
  "model": "aila-ocr",
  "messages": [
    {
      "role": "user",
      "content": "Extract text"
    }
  ],
  "max_tokens": 1024,
  "temperature": 0.7,
  "top_p": 0.9,
  "top_k": 50,
  "seed": 42,
  "stream": false
}
```

**Vision Request (Multi-Modal):**
```json
{
  "model": "aila-ocr",
  "messages": [
    {
      "role": "user",
      "content": [
        {"type": "text", "text": "Extract ID information"},
        {"type": "image_url", "image_url": {"url": "data:image/jpeg;base64,..."}}
      ]
    }
  ],
  "max_tokens": 1000,
  "temperature": 0.7,
  "top_p": 0.9,
  "top_k": 50
}
```

**Supported Parameters:**
- `model` (required) - Model key
- `messages` (required) - Array of messages
- `temperature` (0-2) - Sampling temperature
- `max_tokens` - Maximum tokens to generate
- `top_p` (0-1) - Nucleus sampling
- `top_k` - Top-K sampling
- `min_p` - Minimum probability threshold
- `seed` - Deterministic sampling
- `frequency_penalty` (-2 to 2) - Frequency penalty
- `presence_penalty` (-2 to 2) - Presence penalty
- `repetition_penalty` - Repetition penalty
- `stop` - Stop sequences
- `stream` - Stream response
- `logprobs` - Return log probabilities
- `top_logprobs` - Number of logprobs to return
- `response_format` - Output format (json_object, text)
- `tools` - Function calling tools
- `tool_choice` - Tool selection strategy

**Response:**
```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "aila-ocr",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "ROYAUME DU MAROC\nCARTE NATIONALE D'IDENTITE\n..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 4979,
    "completion_tokens": 135,
    "total_tokens": 5114
  },
  "system_fingerprint": null
}
```

**Note:** Response `model` field returns the requested model key, not the backend model type.

Headers:
| Header | Required | Description |
|--------|----------|-------------|
| Content-Type | Yes | application/json |
| Authorization | When REQUIRE_AUTH=true | Bearer aila_live_xxx |

### Text Completions

**POST** `/api/v1/completions`

Request:
```json
{
  "model": "aila-ocr",
  "prompt": "Extract text from:",
  "max_tokens": 500
}
```

### Embeddings

**POST** `/api/v1/embeddings`

Request:
```json
{
  "model": "text-embedding-ada-002",
  "input": "The quick brown fox"
}
```

### Health Check

**GET** `/health`

Response: `Working!`

### Metrics

**GET** `/metrics`

Response: Prometheus metrics format

---

## Management API (Port 8080)

### OAuth Services

#### Create OAuth Service

**POST** `/api/v1/management/oauth-services`

Request:
```json
{
  "application_service": "OCR-Invoice-Service",
  "rate_limit_per_minute": 100,
  "allowed_models": ["aila-ocr", "gpt-4"],
  "metadata": {"team": "backend"},
  "token_expiration_hours": 24
}
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "application_service": "OCR-Invoice-Service",
  "application_id": "550e8400-e29b-41d4-a716-446655440001",
  "api_key": "aila_live_aBcDeFgHiJkLmNoPqRsTuVwXyZ123456",
  "api_secret": "aila_secret_aBcDeFgHiJkLmNoPqRsTuVwXyZ123456",
  "rate_limit_per_minute": 100,
  "allowed_models": ["aila-ocr", "gpt-4"],
  "token_expiration_hours": 24,
  "expires_at": "2025-06-02T00:00:00Z",
  "created_at": "2025-06-01T00:00:00Z"
}
```

#### List OAuth Services

**GET** `/api/v1/management/oauth-services`

Response:
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "application_service": "OCR-Invoice-Service",
    "application_id": "550e8400-e29b-41d4-a716-446655440001",
    "api_key_masked": "aila_live_***",
    "rate_limit_per_minute": 100,
    "allowed_models": ["aila-ocr"],
    "metadata": {},
    "is_active": true,
    "token_expiration_hours": 24,
    "expires_at": "2025-06-02T00:00:00Z",
    "is_expired": false,
    "created_at": "2025-06-01T00:00:00Z",
    "updated_at": "2025-06-01T00:00:00Z",
    "last_used_at": "2025-06-01T12:00:00Z"
  }
]
```

#### Get OAuth Service

**GET** `/api/v1/management/oauth-services/{id}`

#### Update OAuth Service

**PUT** `/api/v1/management/oauth-services/{id}`

Request:
```json
{
  "application_service": "Updated-Service-Name",
  "rate_limit_per_minute": 200,
  "allowed_models": ["aila-ocr"],
  "is_active": true,
  "token_expiration_hours": 48
}
```

#### Delete OAuth Service

**DELETE** `/api/v1/management/oauth-services/{id}`

Response: 204 No Content

#### Rotate API Key

**POST** `/api/v1/management/oauth-services/{id}/rotate-key`

Response:
```json
{
  "new_api_key": "aila_live_newKeyHere123456789",
  "expires_at": "2025-07-01T00:00:00Z",
  "rotated_at": "2025-06-01T00:00:00Z"
}
```

#### Regenerate Token

**POST** `/api/v1/management/oauth-services/{id}/regenerate-token`

Response:
```json
{
  "new_api_key": "aila_live_regeneratedKey123456",
  "expires_at": "2025-07-01T00:00:00Z",
  "regenerated_at": "2025-06-01T00:00:00Z"
}
```

#### Lookup Client

**GET** `/api/v1/management/oauth-services/lookup`

Query Parameters:
| Parameter | Type | Description |
|-----------|------|-------------|
| application_service | string | Application service name to search |
| application_id | string | Application ID to search |

---

### Providers

#### Create Provider

**POST** `/api/v1/management/providers`

**OpenAI/vLLM Provider:**
```json
{
  "name": "vllm-aila",
  "provider_type": "OpenAI",
  "config": {
    "api_key": {"type": "literal", "value": "EMPTY"},
    "base_url": "http://vllm:8000/v1"
  },
  "enabled": true
}
```

**Anthropic Provider:**
```json
{
  "name": "anthropic-prod",
  "provider_type": "Anthropic",
  "config": {
    "api_key": {"type": "environment", "variable_name": "ANTHROPIC_API_KEY"}
  },
  "enabled": true
}
```

**Azure OpenAI Provider:**
```json
{
  "name": "azure-prod",
  "provider_type": "Azure",
  "config": {
    "api_key": {"type": "literal", "value": "xxx"},
    "resource_name": "my-resource",
    "api_version": "2024-02-01"
  },
  "enabled": true
}
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "vllm-aila",
  "provider_type": "OpenAI",
  "enabled": true,
  "created_at": "2026-02-21T07:00:00Z",
  "updated_at": "2025-06-01T00:00:00Z"
}
```

#### List Providers

**GET** `/api/v1/management/providers`

#### Get Provider

**GET** `/api/v1/management/providers/{id}`

#### Update Provider

**PUT** `/api/v1/management/providers/{id}`

#### Delete Provider

**DELETE** `/api/v1/management/providers/{id}`

---

### Model Definitions

#### Create Model

**POST** `/api/v1/management/model-definitions`

**Request:**
```json
{
  "key": "aila-ocr",
  "model_type": "allenai/olmOCR-2-7B-1025-FP8",
  "provider_id": "550e8400-e29b-41d4-a716-446655440000",
  "enabled": true
}
```

**Response:**
```json
{
  "id": "660e8400-e29b-41d4-a716-446655440000",
  "key": "aila-ocr",
  "model_type": "allenai/olmOCR-2-7B-1025-FP8",
  "provider_id": "550e8400-e29b-41d4-a716-446655440000",
  "enabled": true,
  "created_at": "2026-02-21T07:00:00Z"
}
```

#### List Models

**GET** `/api/v1/management/model-definitions`

#### Get Model

**GET** `/api/v1/management/model-definitions/{id}`

#### Update Model

**PUT** `/api/v1/management/model-definitions/{id}`

#### Delete Model

**DELETE** `/api/v1/management/model-definitions/{id}`

---

### Pipelines

#### Create Pipeline

**POST** `/api/v1/management/pipelines`

**Single Model Pipeline:**
```json
{
  "name": "ocr-pipeline",
  "pipeline_type": "Chat",
  "description": "OCR pipeline for vision models",
  "plugins": [{
    "plugin_type": "model-router",
    "config_data": {"models": [{"key": "aila-ocr", "priority": 0}]},
    "enabled": true,
    "order_in_pipeline": 0
  }],
  "enabled": true
}
```

**Multi-Model Routing Pipeline:**
```json
{
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
}
```

**Pipeline with Tracing:**
```json
{
  "name": "traced-pipeline",
  "pipeline_type": "Chat",
  "plugins": [
    {
      "plugin_type": "tracing",
      "config_data": {
        "endpoint": "https://cloud.langfuse.com",
        "public_key": {"type": "environment", "variable_name": "LANGFUSE_PUBLIC_KEY"},
        "secret_key": {"type": "environment", "variable_name": "LANGFUSE_SECRET_KEY"}
      },
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
}
```

**Response:**
```json
{
  "id": "770e8400-e29b-41d4-a716-446655440000",
  "name": "ocr-pipeline",
  "pipeline_type": "Chat",
  "description": "OCR pipeline for vision models",
  "plugins": [...],
  "enabled": true,
  "created_at": "2026-02-21T07:00:00Z"
}
```

#### List Pipelines

**GET** `/api/v1/management/pipelines`

#### Get Pipeline

**GET** `/api/v1/management/pipelines/{id}`

#### Update Pipeline

**PUT** `/api/v1/management/pipelines/{id}`

#### Delete Pipeline

**DELETE** `/api/v1/management/pipelines/{id}`

---

## Error Responses

### Authentication Errors

**401 Unauthorized - Missing API Key**
```json
{
  "error": {
    "message": "Authorization header is required",
    "type": "authentication_error",
    "code": "missing_api_key"
  }
}
```

**401 Unauthorized - Invalid API Key**
```json
{
  "error": {
    "message": "Invalid API key provided",
    "type": "authentication_error",
    "code": "invalid_api_key"
  }
}
```

**401 Unauthorized - Expired API Key**
```json
{
  "error": {
    "message": "API key has expired. Please regenerate your token using POST /api/v1/management/oauth-services/{id}/regenerate-token",
    "type": "authentication_error",
    "code": "expired_api_key"
  }
}
```

### Permission Errors

**403 Forbidden - Inactive Client**
```json
{
  "error": {
    "message": "Client account is inactive",
    "type": "permission_error",
    "code": "inactive_client"
  }
}
```

### Rate Limit Errors

**429 Too Many Requests**
```json
{
  "error": {
    "message": "Rate limit exceeded. Limit: 100 req/min",
    "type": "rate_limit_error",
    "code": "rate_limit_exceeded"
  }
}
```

### Validation Errors

**400 Bad Request**
```json
{
  "error": {
    "message": "Invalid request body",
    "type": "validation_error",
    "code": "invalid_request"
  }
}
```

### Not Found Errors

**404 Not Found**
```json
{
  "error": {
    "message": "OAuth client not found",
    "type": "not_found_error",
    "code": "not_found"
  }
}
```

### Conflict Errors

**409 Conflict**
```json
{
  "error": {
    "message": "OAuth client with name 'My App' already exists",
    "type": "conflict_error",
    "code": "conflict"
  }
}
```

---

## Provider Types

| Type | Description |
|------|-------------|
| openai | OpenAI API (also vLLM, AILA OCR) |
| anthropic | Anthropic Claude |
| azure | Azure OpenAI |
| vertexai | Google VertexAI |
| bedrock | AWS Bedrock |

## Pipeline Types

| Type | Description |
|------|-------------|
| chat | Chat completions |
| completion | Text completions |
| embeddings | Text embeddings |

## Plugin Types

| Type | Config |
|------|--------|
| logging | {"level": "info"} |
| tracing | {"endpoint": "...", "api_key": "..."} |
| model-router | {"models": ["model1", "model2"]} |
