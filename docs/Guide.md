# vLLM OCR Model Setup Guide

This guide explains how to configure the AILA OCR model (powered by vLLM) with the Hid-OAuth Gateway.

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

## Overview

The AILA OCR model uses **vLLM inference** which is **OpenAI API compatible**. This means we can register it as an OpenAI-type provider with a custom `base_url` pointing to the vLLM endpoint.

## Model Details

| Property | Value |
|----------|-------|
| **Model Name** | `aila-ocr` |
| **Underlying Model** | `allenai/olmOCR-2-7B-1025-FP8` |
| **Provider Type** | OpenAI-compatible (vLLM) |
| **Endpoint** | `http://vllm-inference76/v1` |
| **Capabilities** | OCR, Document Text Extraction |

## Prerequisites

1. Hid-OAuth running in **Database Mode** (`HID_OAUTH_MODE=database`)
2. PostgreSQL database configured
3. Management API accessible on port 8080

## Quick Setup with Makefile

```bash
# Build and start hub in database mode
make quickstart

# Register AILA OCR provider, model, and pipeline
make register-aila-ocr

# Test the OCR model
make test-ocr
```

## Manual Setup via API

### Step 1: Register the vLLM Provider

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

### Step 2: Register the OCR Model

```bash
curl -X POST http://localhost:8080/api/v1/management/model-definitions \
  -H "Content-Type: application/json" \
  -d '{
    "model_name": "aila-ocr",
    "model_type": "allenai/olmOCR-2-7B-1025-FP8",
    "provider_name": "aila-vllm",
    "enabled": true
  }'
```

### Step 3: Create a Pipeline with the Model

```bash
curl -X POST http://localhost:8080/api/v1/management/pipelines \
  -H "Content-Type: application/json" \
  -d '{
    "name": "default",
    "pipeline_type": "chat",
    "plugins": [
      {"type": "logging", "config": {"level": "info"}},
      {"type": "model-router", "config": {"models": ["aila-ocr"]}}
    ],
    "enabled": true
  }'
```

## Using the OCR Model

### Basic Text Extraction

```bash
curl -X POST http://localhost:3000/api/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "aila-ocr",
    "messages": [
      {
        "role": "user",
        "content": [
          {"type": "text", "text": "Extract all readable text from this document."},
          {"type": "image_url", "image_url": {"url": "https://example.com/document.jpg"}}
        ]
      }
    ],
    "max_tokens": 1024
  }'
```

### With OAuth Authentication

If OAuth is enabled (`REQUIRE_AUTH=true`), include the Bearer token:

```bash
curl -X POST http://localhost:3000/api/v1/chat/completions \
  -H "Authorization: Bearer hid_live_your_api_key_here" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "aila-ocr",
    "messages": [
      {
        "role": "user",
        "content": [
          {"type": "text", "text": "Extract all readable text from this document."},
          {"type": "image_url", "image_url": {"url": "https://example.com/document.jpg"}}
        ]
      }
    ],
    "max_tokens": 1024
  }'
```

## Architecture

```mermaid
flowchart TD
    subgraph Client["Client Layer"]
        REQ["Client Request<br/>model: aila-ocr"]
    end

    subgraph Gateway["Hid-OAuth Gateway :3000"]
        AUTH["1. Auth Middleware<br/>(if REQUIRE_AUTH=true)"]
        PIPE["2. Pipeline Router<br/>default pipeline"]
        MODEL["3. Model Router<br/>aila-ocr model"]
        PROV["4. Provider Lookup<br/>aila-vllm (OpenAI-compatible)"]
        
        AUTH --> PIPE --> MODEL --> PROV
    end

    subgraph VLLM["vLLM Inference Server"]
        ENDPOINT["http://vllm-inference76/v1"]
        MODELINFO["Model: allenai/olmOCR-2-7B-1025-FP8<br/>API: OpenAI-compatible"]
    end

    REQ --> AUTH
    PROV --> ENDPOINT
```

**Request Flow:**

```mermaid
sequenceDiagram
    participant Client
    participant Auth as Auth Middleware
    participant Gateway as Hid-OAuth Gateway
    participant vLLM as vLLM Server

    Client->>+Auth: POST /api/v1/chat/completions
    Auth->>Auth: Validate Bearer Token
    Auth->>Auth: Check Token Expiration
    Auth->>+Gateway: Authenticated Request
    Gateway->>Gateway: Route to Pipeline
    Gateway->>Gateway: Lookup Model Provider
    Gateway->>+vLLM: Forward to vLLM
    vLLM-->>-Gateway: OCR Response
    Gateway-->>-Auth: Response
    Auth-->>-Client: JSON Response
```

## Configuration Options

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HID_OAUTH_MODE` | Must be `database` for API registration | `yaml` |
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `REQUIRE_AUTH` | Enable OAuth authentication | `false` |
| `RUST_LOG` | Logging level | `warn` |

### Provider Configuration

The vLLM provider uses the `openai` provider type with custom `base_url`:

```json
{
  "name": "aila-vllm",
  "provider_type": "openai",
  "config": {
    "api_key": {"type": "literal", "value": "<your-vllm-api-key>"},
    "base_url": "http://<vllm-host>/v1"
  }
}
```

## Troubleshooting

### Model Not Found

```json
{"error": {"message": "Model 'aila-ocr' not found", "type": "invalid_request_error"}}
```

**Solution**: Ensure the model is registered and the pipeline includes it in the model-router.

### Provider Connection Error

```json
{"error": {"message": "Failed to connect to provider", "type": "server_error"}}
```

**Solution**: Verify the vLLM endpoint is accessible from the hub container.

### Authentication Error

```json
{"error": {"message": "Authorization header is required", "type": "authentication_error"}}
```

**Solution**: Include `Authorization: Bearer <api_key>` header when `REQUIRE_AUTH=true`.

## Direct vLLM API Testing

Test the vLLM endpoint directly:

```bash
# Get model info
curl -X GET http://vllm-inference76/v1/models \
  -H "Authorization: Bearer YOUR_HF_TOKEN"

# Chat completion
curl -X POST http://vllm-inference76/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_HF_TOKEN" \
  -d '{
    "model": "allenai/olmOCR-2-7B-1025-FP8",
    "messages": [
      {
        "role": "user",
        "content": [
          {"type": "text", "text": "Extract all readable text from this document."},
          {"type": "image_url", "image_url": {"url": "https://example.com/document.jpg"}}
        ]
      }
    ],
    "max_tokens": 1024
  }'
```
