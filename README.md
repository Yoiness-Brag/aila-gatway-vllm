<p align="center">
  <img src="img/orc-logo.png" alt="AILA OCR Logo" />
</p>

# AILA-OAuth Gateway
LLM gateway written in Rust with OAuth authentication, multi-provider support, and dynamic configuration.

## Features

- OAuth service authentication with configurable token expiration
- Multi-provider support: OpenAI, Anthropic, Azure OpenAI, vLLM, AWS Bedrock, Google VertexAI
- OpenAI-compatible API endpoints
- Multi-modal vision API support (text + image)
- Full parameter support: top_k, top_p, seed, repetition_penalty, etc.
- Database mode with dynamic configuration via Management API
- Per-service rate limiting and usage tracking
- OpenTelemetry tracing support (Langfuse, Jaeger, etc.)
- High performance async Rust implementation

## Compatibility Matrix

| Parameter | OpenAI | vLLM | Anthropic | Bedrock | VertexAI | Gateway |
|-----------|--------|------|-----------|---------|----------|----------|
| **Core** |
| model | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| messages | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| temperature | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| max_tokens | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| **Sampling** |
| top_p | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| top_k | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ |
| min_p | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ |
| seed | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ |
| **Penalties** |
| frequency_penalty | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ |
| presence_penalty | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ |
| repetition_penalty | ✗ | ✓ | ✗ | ✗ | ✗ | ✓ |
| **Vision** |
| image_url | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ |
| **Tools** |
| tools | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ |
| tool_choice | ✓ | ✓ | ✓ | ✗ | ✓ | ✓ |
| **Advanced** |
| logprobs | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ |
| logit_bias | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ |
| response_format | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ |
| extra_body | ✓ | ✓ | ✗ | ✗ | ✗ | ✓ |


## Quick Start

```bash
# 1. Start PostgreSQL and run migrations
docker compose up -d postgres
docker compose up migrations

# 2. Start gateway
docker compose -f docker-compose-aila-oauth.yml up -d'
```

## Example Usage

### LangChain with Structured Output

```python
from langchain_openai import ChatOpenAI
from pydantic import BaseModel, Field
import base64

class OCRResult(BaseModel):
    full_name: str = Field(description="Full name from ID")
    birth_date: str = Field(description="Birth date")
    id_number: str = Field(description="ID number")

llm = ChatOpenAI(
    base_url="http://localhost:3000/api/v1",
    api_key="aila_live_xxx",
    model="aila-ocr",
    temperature=0.7
)

llm_structured = llm.with_structured_output(OCRResult)
image_b64 = base64.b64encode(open("id.jpg", "rb").read()).decode()

result = llm_structured.invoke([{
    "role": "user",
    "content": [
        {"type": "text", "text": "Extract ID information"},
        {"type": "image_url", "image_url": {"url": f"data:image/jpeg;base64,{image_b64}"}}
    ]
}])
```

### OpenAI Java SDK

```java
OpenAIClient client = OpenAIOkHttpClient.builder()
    .baseUrl("http://localhost:3000/api/v1")
    .apiKey("hid_live_xxx")
    .build();

byte[] imageBytes = Files.readAllBytes(Paths.get("id.jpg"));
String base64Image = Base64.getEncoder().encodeToString(imageBytes);

ChatCompletionCreateParams params = ChatCompletionCreateParams.builder()
    .model("aila-ocr")
    .addMessage(ChatCompletionUserMessageParam.builder()
        .content(List.of(
            ChatCompletionContentPartTextParam.builder()
                .text("Extract ID information")
                .build(),
            ChatCompletionContentPartImageParam.builder()
                .imageUrl(ImageUrl.builder()
                    .url("data:image/jpeg;base64," + base64Image)
                    .build())
                .build()
        ))
        .build())
    .temperature(0.7)
    .build();

ChatCompletion completion = client.chat().completions().create(params);
```

### curl with Vision

```bash
IMAGE_B64=$(base64 -w 0 id.jpg)

curl -X POST http://localhost:3000/api/v1/chat/completions \
  -H "Authorization: Bearer aila_live_xxx" \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"aila-ocr\",
    \"messages\": [{
      \"role\": \"user\",
      \"content\": [
        {\"type\": \"text\", \"text\": \"Extract ID information\"},
        {\"type\": \"image_url\", \"image_url\": {\"url\": \"data:image/jpeg;base64,$IMAGE_B64\"}}
      ]
    }],
    \"temperature\": 0.7,
    \"top_p\": 0.9,
    \"top_k\": 50
  }"
```

## Configuration

### Environment Variables

```bash
AILA_OAUTH_MODE=database
DATABASE_URL=postgresql://user:pass@localhost:5432/db
REQUIRE_AUTH=true
DB_POLL_INTERVAL_SECONDS=30
```

## API Schema Guide

### OAuth Service Schema

**Create OAuth Service**
```bash
curl -X POST http://localhost:8080/api/v1/management/oauth-services \
  -H "Content-Type: application/json" \
  -d '{
    "application_service": "OCR-Service",
    "rate_limit_per_minute": 100,
    "token_expiration_hours": 24,
    "allowed_models": ["aila-ocr", "aila-ocr-2"],
    "metadata": {"env": "production", "team": "ai"}
  }'
```

**Response:**
```json
{
  "id": "<uuid>",
  "application_service": "OCR-Service",
  "application_id": "<uuid>",
  "api_key": "aila_live_xxx",
  "rate_limit_per_minute": 100,
  "token_expiration_hours": 24,
  "allowed_models": ["aila-ocr", "aila-ocr-2"],
  "is_active": true,
  "created_at": "2026-02-21T07:00:00Z"
}
```

### Management API

Port 8080 provides REST API for managing providers, models, and pipelines.

#### Create Provider

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

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "vllm-aila",
  "provider_type": "OpenAI",
  "enabled": true,
  "created_at": "2026-02-21T07:00:00Z"
}
```

#### Create Model Definition

Use the `id` from provider response as `provider_id`:

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

#### Create Pipeline (Single Model)

```bash
curl -X POST http://localhost:8080/api/v1/management/pipelines \
  -H "Content-Type: application/json" \
  -d '{
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
  }'
```

#### Create Pipeline (Multi-Model Routing)

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

### OpenTelemetry Tracing

Add tracing plugin to pipelines to send traces to external backends:

```bash
curl -X POST http://localhost:8080/api/v1/management/pipelines \
  -H "Content-Type: application/json" \
  -d '{
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
    ]
  }'
```

Supports any OTLP-compatible backend (Langfuse, Jaeger, etc.). See `docs/guides/langfuse/langfuse-integration.md`.

---

## System Architecture

<p align="center">
  <img src="img/arch.png" alt="System Architecture Diagram" />
</p>

**Architecture Overview:**

The architecture diagram illustrates the complete request flow through the AILA-OAuth Gateway, showing how client requests are authenticated via OAuth, routed through pipelines, and forwarded to various LLM providers (OpenAI, Anthropic, Azure, vLLM/AILA OCR). 

**Key Components:**
- **Port 3000**: LLM Gateway - Handles authenticated inference requests
- **Port 8080**: Management API - Dynamic configuration and monitoring
- **OAuth Layer**: Client authentication and rate limiting
- **Pipeline System**: Flexible routing with model fallback support
- **Multi-Provider Support**: Unified interface for all LLM backends

---

<!-- ## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details. -->

## Documentation

- `docs/guides/database/management_db.md` - Management API reference
- `docs/guides/database/re-migartion.md` - Database migrations
- `docs/guides/langfuse/langfuse-integration.md` - OpenTelemetry tracing setup

## Support

Email: youness.elBrag@aila.sa

---

## Management Panel

<p align="center">
  <img src="img/panle.png" alt="Management Panel Interface" />
</p>

**Management Panel Overview:**

The management panel provides a comprehensive visual interface for configuring and monitoring the AILA-OAuth Gateway. Administrators can dynamically manage the entire gateway configuration without requiring restarts.

**Panel Features:**
- **Provider Management**: Configure and monitor LLM provider connections
- **Model Definitions**: Map user-facing model keys to actual provider models
- **Pipeline Configuration**: Create and manage request routing pipelines
- **OAuth Services**: Generate and manage API keys for client applications
- **Real-time Monitoring**: Track usage, rate limits, and system health
- **REST API Access**: All operations available via port 8080 Management API

The panel enables complete control over the gateway's behavior, allowing teams to adapt to changing requirements, add new models, and manage access control dynamically.
