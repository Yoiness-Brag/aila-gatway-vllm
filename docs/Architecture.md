# AILA-OAuth Architecture

## System Overview

The AILA-OAuth Gateway is a high-performance LLM proxy written in Rust that provides unified access to multiple LLM providers with built-in OAuth client authentication, rate limiting, and usage tracking.

**Key Components:**
- **Client Layer**: Applications that consume the LLM API using OAuth tokens
- **Gateway Layer (Port 3000)**: Handles authentication, routing, and request processing
- **Management Layer (Port 8080)**: REST API for configuration management
- **Data Layer**: PostgreSQL database for persistent storage
- **Provider Layer**: Supported LLM backends (OpenAI, Anthropic, Azure, vLLM/AILA OCR)

## Why OAuth Authentication Layer?

### Architectural Reasoning

The OAuth authentication layer serves as a **critical security and governance boundary** between client applications and backend LLM providers. This separation of concerns provides multiple architectural benefits:

#### 1. **Security Isolation**
- **Problem**: Exposing provider API keys directly to client applications creates security risks
- **Solution**: OAuth layer acts as a proxy - clients never see provider credentials
- **Benefit**: Provider API keys (e.g., vLLM HuggingFace token) remain secure on the gateway

#### 2. **Multi-Tenancy & Client Management**
- **Problem**: Multiple applications need access to the same LLM providers
- **Solution**: Each client gets unique OAuth credentials (`aila_live_xxx`) with individual:
  - Rate limits (`rate_limit_per_minute`)
  - Allowed models (`allowed_models` JSONB array)
  - Token expiration (`token_expiration_hours`)
  - Usage tracking and audit logs
- **Benefit**: Granular control per application without modifying provider configurations

#### 3. **Provider Abstraction**
- **Problem**: Different providers use different authentication mechanisms
- **Solution**: OAuth layer provides unified authentication interface
- **Benefit**: Clients use one token format regardless of backend provider (OpenAI, vLLM, Anthropic, etc.)

#### 4. **Request Flow Separation**

```
Client Request → OAuth Validation → Model Routing → Provider Authentication → LLM Backend
     ↓                  ↓                  ↓                    ↓                  ↓
Bearer Token    Hash Validation    Model Key Lookup    Provider API Key    Actual Model
aila_live_xxx    (Database)         aila-ocr → model    HF Token           allenai/olmOCR
```

**Critical Distinction:**
- **OAuth Token** (`aila_live_xxx`): Authenticates the **client application** to access the gateway
- **Provider API Key** (e.g., HuggingFace token): Authenticates the **gateway** to access the LLM provider
- **Model Key** (`aila-ocr`): Routes request to correct model definition in pipeline
- **Model Type** (`allenai/olmOCR-2-7B-1025-FP8`): Actual model name sent to provider

#### 5. **Rate Limiting & Cost Control**
- **Problem**: Uncontrolled LLM usage can lead to cost overruns
- **Solution**: Per-client rate limits enforced at OAuth layer
- **Implementation**: Database-backed counter checks requests per minute
- **Benefit**: Prevent abuse and control costs per application

#### 6. **Usage Tracking & Auditing**
- **Problem**: Need visibility into which applications use which models
- **Solution**: `hub_llmgateway_client_usage_logs` table tracks every request
- **Data Captured**:
  - `oauth_service_id`: Which client made the request
  - `endpoint`: Which API endpoint was called
  - `model`: Which model was used
  - `tokens_used`: Token consumption
  - `latency_ms`: Response time
  - `response_status`: Success/failure
  - `request_timestamp`: When request occurred
- **Benefit**: Complete audit trail for billing, debugging, and optimization

#### 7. **Dynamic Configuration Without Downtime**
- **Problem**: Changing provider credentials requires application restart
- **Solution**: OAuth clients and provider configs stored in database
- **Benefit**: Update provider API keys, add new clients, modify rate limits without gateway restart

```mermaid
graph TB
    subgraph "Client Layer"
        C1[Client Application]
        C2[OAuth Client]
    end

    subgraph "Gateway Layer - Port 3000"
        AUTH[Auth Middleware]
        GW[Gateway Router]
        PIPE[Pipeline System]
    end

    subgraph "Management Layer - Port 8080"
        MGMT[Management API]
        OAUTH[OAuth Client Service]
        PROV[Provider Service]
        MODEL[Model Service]
        PIPELINE[Pipeline Service]
    end

    subgraph "Data Layer"
        PG[(PostgreSQL)]
    end

    subgraph "Provider Layer"
        OPENAI[OpenAI]
        ANTHROPIC[Anthropic]
        AZURE[Azure OpenAI]
        VERTEXAI[VertexAI]
        BEDROCK[Bedrock]
        VLLM[vLLM/AILA OCR]
    end

    C1 -->|Bearer Token| AUTH
    C2 -->|Management| MGMT
    AUTH -->|Validated| GW
    GW --> PIPE
    PIPE --> OPENAI
    PIPE --> ANTHROPIC
    PIPE --> AZURE
    PIPE --> VERTEXAI
    PIPE --> BEDROCK
    PIPE --> VLLM

    MGMT --> OAUTH
    MGMT --> PROV
    MGMT --> MODEL
    MGMT --> PIPELINE

    OAUTH --> PG
    PROV --> PG
    MODEL --> PG
    PIPELINE --> PG
    AUTH --> PG
```

## Complete Request Flow with OAuth Layer

The request flow illustrates how an authenticated client request travels through the gateway to reach the LLM provider. Each step performs validation and transformation before forwarding the request.

### Detailed Processing Steps

#### Step 1: Client Request

**Text Request:**
```json
POST http://localhost:3000/api/v1/chat/completions
Authorization: Bearer aila_live_xxx
Content-Type: application/json

{
  "model": "aila-ocr",
  "messages": [{"role": "user", "content": "Extract text"}],
  "max_tokens": 1024,
  "temperature": 0.7,
  "top_p": 0.9,
  "top_k": 50
}
```

**Vision Request (Multi-Modal):**
```json
POST http://localhost:3000/api/v1/chat/completions
Authorization: Bearer aila_live_xxx
Content-Type: application/json

{
  "model": "aila-ocr",
  "messages": [{
    "role": "user",
    "content": [
      {"type": "text", "text": "Extract ID information"},
      {"type": "image_url", "image_url": {"url": "data:image/jpeg;base64,..."}}
    ]
  }],
  "max_tokens": 1000,
  "temperature": 0.7,
  "top_p": 0.9,
  "top_k": 50
}
```


#### Step 2: LLM Provider Request
```json
POST http://vllm-inference/v1/chat/completions
Authorization: Bearer YOUR_HF_TOKEN
Content-Type: application/json

{
  "model": "allenai/olmOCR-2-7B-1025-FP8",
  "messages": [{"role": "user", "content": "Extract text from image"}],
  "max_tokens": 1024,
  "temperature": 0.7,
  "top_p": 0.9,
  "top_k": 50
}
```

#### Step 3: Response Flow
- vLLM → Provider → Pipeline → Gateway → Client
- Usage logged to `hub_llmgateway_client_usage_logs`
- `last_used_at` updated in `hub_llmgateway_oauth_clients`

### Key Architectural Insights

**Two-Layer Authentication:**
1. **Client → Gateway**: OAuth token (`aila_live_xxx`) validates client application
2. **Gateway → Provider**: Provider API key (e.g., HuggingFace token) authenticates gateway

**Model Name Translation:**
- Client sends: `"model": "aila-ocr"` (friendly key)
- Gateway resolves: `aila-ocr` → `allenai/olmOCR-2-7B-1025-FP8`
- Provider receives: `"model": "allenai/olmOCR-2-7B-1025-FP8"` (actual model)
- Response returns: `"model": "aila-ocr"` (preserves requested key)

**Why This Design?**
- **Abstraction**: Clients don't need to know actual model names
- **Flexibility**: Change backend model without updating clients
- **Security**: Provider credentials never exposed to clients
- **Control**: Enforce which models each client can access

```mermaid
sequenceDiagram
    participant Client
    participant AuthMiddleware
    participant Gateway
    participant Pipeline
    participant Provider
    participant LLM

    Client->>AuthMiddleware: POST /api/v1/chat/completions
    Note over AuthMiddleware: Extract Bearer Token
    AuthMiddleware->>AuthMiddleware: Validate API Key Hash
    AuthMiddleware->>AuthMiddleware: Check Token Expiration
    AuthMiddleware->>AuthMiddleware: Check Rate Limit
    AuthMiddleware->>Gateway: Request + AuthenticatedClient
    Gateway->>Pipeline: Route to Pipeline
    Pipeline->>Provider: Forward Request
    Provider->>LLM: API Call
    LLM-->>Provider: Response
    Provider-->>Pipeline: Response
    Pipeline-->>Gateway: Response
    Gateway-->>Client: JSON Response
```

## OAuth Authentication Flow

The OAuth flow implements a secure client authentication system with configurable token expiration and automatic regeneration capability.

**Security Features:**
- **Token Format**: `aila_live_<32_random_chars>` - easily identifiable prefix
- **Configurable Expiration**: Token expiration is configurable via `token_expiration_hours` (default: 24h)
- **Token Regeneration**: Clients can regenerate expired tokens via API
- **Application ID**: Auto-generated unique identifier (`app_<16_chars>`) for each application
- **Client Lookup**: Check if client exists by application_service or application_id
- **Rate Limiting**: Per-client configurable request limits
- **Usage Logging**: All requests are logged for auditing

```mermaid
sequenceDiagram
    participant Admin
    participant ManagementAPI
    participant Database
    participant Client
    participant Gateway

    Admin->>ManagementAPI: POST /oauth-services
    Note over ManagementAPI: Generate API Key (aila_live_xxx)
    Note over ManagementAPI: Hash API Key
    ManagementAPI->>Database: Store Client + Hash
    ManagementAPI-->>Admin: API Key (shown once)

    Admin->>Client: Provide API Key

    Client->>Gateway: Request + Bearer Token
    Gateway->>Database: Validate Hash
    Database-->>Gateway: Client Info
    Note over Gateway: Check expires_at < NOW()
    Note over Gateway: Check is_active = true
    Note over Gateway: Check rate_limit
    Gateway-->>Client: Response
```

## Token Expiration Flow

```mermaid
stateDiagram-v2
    [*] --> Created: POST /oauth-services
    Created --> Active: API Key Generated
    Active --> Active: Valid Requests
    Active --> Expired: Token Expired
    Expired --> Active: POST /regenerate-token
    Active --> Inactive: Deactivated
    Inactive --> Active: Reactivated
```

## Database Schema

```mermaid
erDiagram
    hub_llmgateway_oauth_clients {
        uuid id PK
        varchar application_service
        uuid application_id UK
        varchar api_key_hash
        varchar api_secret_hash
        int rate_limit_per_minute
        jsonb allowed_models
        jsonb metadata
        boolean is_active
        int token_expiration_hours
        timestamptz expires_at
        timestamptz created_at
        timestamptz updated_at
        timestamptz last_used_at
    }

    hub_llmgateway_client_usage_logs {
        uuid id PK
        uuid oauth_service_id FK
        varchar endpoint
        varchar model
        int tokens_used
        int latency_ms
        int response_status
        text error_message
        timestamptz request_timestamp
    }

    hub_llmgateway_providers {
        uuid id PK
        varchar name UK
        varchar provider_type
        jsonb config
        boolean enabled
        timestamptz created_at
        timestamptz updated_at
    }

    hub_llmgateway_model_definitions {
        uuid id PK
        varchar key UK
        uuid provider_id FK
        varchar model_name
        jsonb config
        boolean enabled
        timestamptz created_at
        timestamptz updated_at
    }

    hub_llmgateway_pipelines {
        uuid id PK
        varchar name UK
        varchar route_path
        varchar model_key
        boolean enabled
        timestamptz created_at
        timestamptz updated_at
    }

    hub_llmgateway_pipeline_plugin_configs {
        uuid id PK
        uuid pipeline_id FK
        varchar plugin_type
        jsonb config_data
        boolean enabled
        int order_in_pipeline
        timestamptz created_at
        timestamptz updated_at
    }

    hub_llmgateway_oauth_clients ||--o{ hub_llmgateway_client_usage_logs : logs
    hub_llmgateway_providers ||--o{ hub_llmgateway_model_definitions : has
    hub_llmgateway_pipelines ||--o{ hub_llmgateway_pipeline_plugin_configs : contains
```

## Module Structure

```mermaid
graph TD
    subgraph "src/"
        MAIN[main.rs]
        LIB[lib.rs]
        ROUTES[routes.rs]
        STATE[state.rs]
    end

    subgraph "src/middleware/"
        AUTH_MW[auth.rs]
        MOD_MW[mod.rs]
    end

    subgraph "src/management/"
        MOD_MGMT[mod.rs]
        DTO[dto.rs]
        ERRORS[errors.rs]
    end

    subgraph "src/management/api/routes/"
        OAUTH_ROUTES[oauth_client_routes.rs]
        PROV_ROUTES[provider_routes.rs]
        MODEL_ROUTES[model_definition_routes.rs]
        PIPE_ROUTES[pipeline_routes.rs]
    end

    subgraph "src/management/services/"
        OAUTH_SVC[oauth_client_service.rs]
        PROV_SVC[provider_service.rs]
        MODEL_SVC[model_definition_service.rs]
        PIPE_SVC[pipeline_service.rs]
        CONFIG_SVC[config_provider_service.rs]
    end

    subgraph "src/management/db/repositories/"
        OAUTH_REPO[oauth_client_repository.rs]
        PROV_REPO[provider_repository.rs]
        MODEL_REPO[model_definition_repository.rs]
        PIPE_REPO[pipeline_repository.rs]
    end

    subgraph "src/providers/"
        OPENAI_PROV[openai/]
        ANTHROPIC_PROV[anthropic/]
        AZURE_PROV[azure/]
        VERTEXAI_PROV[vertexai/]
        BEDROCK_PROV[bedrock/]
    end

    MAIN --> LIB
    MAIN --> ROUTES
    MAIN --> STATE
    ROUTES --> AUTH_MW
    AUTH_MW --> OAUTH_SVC
    OAUTH_SVC --> OAUTH_REPO
    OAUTH_REPO --> PG[(PostgreSQL)]
```

## Configuration Modes

```mermaid
graph LR
    subgraph "YAML Mode"
        YAML[config.yaml] --> GW1[Gateway :3000]
    end

    subgraph "Database Mode"
        PG[(PostgreSQL)] --> MGMT[Management API :8080]
        MGMT --> GW2[Gateway :3000]
    end
```

## Provider Integration

```mermaid
graph TD
    subgraph "OpenAI-Compatible Providers"
        OPENAI[OpenAI API]
        VLLM[vLLM Server]
        AILA[AILA OCR]
    end

    subgraph "Native Providers"
        ANTHROPIC[Anthropic]
        AZURE[Azure OpenAI]
        VERTEXAI[VertexAI]
        BEDROCK[AWS Bedrock]
    end

    GW[AILA-OAuth Gateway] --> OPENAI
    GW --> VLLM
    GW --> AILA
    GW --> ANTHROPIC
    GW --> AZURE
    GW --> VERTEXAI
    GW --> BEDROCK
```

## Deployment Architecture

```mermaid
graph TB
    subgraph "Docker Compose"
        PG[PostgreSQL :5432]
        GW[AILA-OAuth Gateway :3000/:8080]
    end

    subgraph "External Services"
        VLLM[vLLM Server]
        OPENAI[OpenAI API]
    end

    CLIENT[Client] --> GW
    GW --> PG
    GW --> VLLM
    GW --> OPENAI
```

## Gateway API Service

The AILA-OAuth Gateway provides an OpenAI-compatible REST API for LLM inference.

### Core Endpoints (Port 3000)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/chat/completions` | POST | Chat completions (streaming supported) |
| `/api/v1/completions` | POST | Text completions |
| `/api/v1/embeddings` | POST | Text embeddings |
| `/health` | GET | Health check |
| `/metrics` | GET | Prometheus metrics |

### Management Endpoints (Port 8080)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/v1/management/oauth-services` | CRUD | OAuth client management |
| `/api/v1/management/providers` | CRUD | Provider configuration |
| `/api/v1/management/model-definitions` | CRUD | Model definitions |
| `/api/v1/management/pipelines` | CRUD | Pipeline configuration |

### Request Processing Flow

1. **Auth Middleware**: Validates Bearer token, checks token expiration
2. **Rate Limiter**: Enforces per-client request limits
3. **Pipeline Router**: Routes to configured pipeline
4. **Model Router**: Selects target model
5. **Provider**: Forwards to LLM backend (vLLM, OpenAI, etc.)

### Authentication

- Bearer token format: `aila_live_<random_string>`
- Token expiration: Configurable via `token_expiration_hours` (default: 24h)
- Regeneration: `POST /oauth-services/{id}/regenerate-token`

## Key Features

| Feature | Description |
|---------|-------------|
| Multi-Provider Support | OpenAI, Anthropic, Azure, VertexAI, Bedrock, vLLM |
| Multi-Modal Vision API | Text + image support for VLMs |
| Full Parameter Support | top_k, top_p, seed, repetition_penalty, etc. |
| OAuth Authentication | Configurable token expiration with regeneration |
| Rate Limiting | Per-client configurable limits |
| Usage Logging | Track requests per client |
| Hot Reload | Dynamic configuration updates |
| OpenAI Compatible | Drop-in replacement for OpenAI API |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| AILA_OAUTH_MODE | yaml or database | yaml |
| DATABASE_URL | PostgreSQL connection | - |
| REQUIRE_AUTH | Enable OAuth | false |
| PORT | Gateway port | 3000 |
| MANAGEMENT_PORT | Management API port | 8080 |
| RUST_LOG | Log level | warn |

## Security Features

| Feature | Implementation |
|---------|----------------|
| API Key Hashing | Double hash with DefaultHasher |
| Token Expiration | Configurable via API (default: 24 hours) |
| Rate Limiting | Database-backed counter |
| Secret Masking | API keys masked in responses |
| Client Deactivation | is_active flag |
