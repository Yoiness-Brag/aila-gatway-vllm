# Langfuse Integration Guide

This gateway supports sending OpenTelemetry traces to any Langfuse instance (Cloud or Self-hosted).

## Prerequisites

1. Access to a Langfuse instance:
   - **Langfuse Cloud**: https://cloud.langfuse.com (free tier available)
   - **Self-hosted**: Langfuse v3.22.0+ with OTLP endpoint enabled

2. Langfuse API credentials:
   - Public Key: `pk-lf-...`
   - Secret Key: `sk-lf-...`

Get these from your Langfuse dashboard: Settings → API Keys

## Configuration

### Step 1: Create Pipeline with Langfuse Tracing

```bash
curl -X POST http://localhost:8080/api/v1/management/pipelines \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-pipeline-with-langfuse",
    "pipeline_type": "Chat",
    "plugins": [
      {
        "plugin_type": "tracing",
        "config_data": {
          "endpoint": "https://cloud.langfuse.com",
          "public_key": {
            "type": "literal",
            "value": "pk-lf-YOUR-PUBLIC-KEY"
          },
          "secret_key": {
            "type": "literal",
            "value": "sk-lf-YOUR-SECRET-KEY"
          }
        },
        "enabled": true,
        "order_in_pipeline": 0
      },
      {
        "plugin_type": "model-router",
        "config_data": {
          "models": [{"key": "your-model-key", "priority": 0}]
        },
        "enabled": true,
        "order_in_pipeline": 1
      }
    ]
  }'
```

### Step 2: Use Environment Variables (Recommended)

For better security, use environment variables:

```bash
# Set in your environment
export USER_LANGFUSE_PUBLIC_KEY="pk-lf-YOUR-PUBLIC-KEY"
export USER_LANGFUSE_SECRET_KEY="sk-lf-YOUR-SECRET-KEY"
```

```bash
curl -X POST http://localhost:8080/api/v1/management/pipelines \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-pipeline-with-langfuse",
    "pipeline_type": "Chat",
    "plugins": [
      {
        "plugin_type": "tracing",
        "config_data": {
          "endpoint": "https://cloud.langfuse.com",
          "public_key": {
            "type": "environment",
            "variable_name": "USER_LANGFUSE_PUBLIC_KEY"
          },
          "secret_key": {
            "type": "environment",
            "variable_name": "USER_LANGFUSE_SECRET_KEY"
          }
        },
        "enabled": true,
        "order_in_pipeline": 0
      },
      {
        "plugin_type": "model-router",
        "config_data": {
          "models": [{"key": "your-model-key", "priority": 0}]
        },
        "enabled": true,
        "order_in_pipeline": 1
      }
    ]
  }'
```

## Supported Langfuse Deployments

### Langfuse Cloud
```json
{
  "endpoint": "https://cloud.langfuse.com"
}
```

### Self-hosted Langfuse
```json
{
  "endpoint": "http://your-langfuse-server:3000"
}
```

### Enterprise Langfuse
```json
{
  "endpoint": "https://langfuse.company.com"
}
```

## Verification

### 1. Check Gateway Logs
```bash
docker logs hid-oauth-gateway | grep "OpenTelemetry"
```

Expected output:
```log
INFO: Initializing Langfuse OtelTracer for pipeline my-pipeline-with-langfuse
DEBUG: OpenTelemetry tracer initialized successfully for endpoint: https://cloud.langfuse.com
```

### 2. Send Test Request
```bash
curl -X POST http://localhost:3000/api/v1/chat/completions \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "your-model-key",
    "messages": [{"role": "user", "content": "Test Langfuse tracing"}],
    "max_tokens": 50
  }'
```

### 3. Check Langfuse Dashboard
1. Log in to your Langfuse instance
2. Navigate to "Traces" section
3. You should see traces with:
   - Operation: `langfuse.chat`
   - Attributes: model, tokens, latency
   - Content: request/response (if enabled)

## Trace Data Captured

The gateway automatically captures:

- Request metadata (model, temperature, max_tokens, etc.)
- Prompt messages (if `TRACE_CONTENT_ENABLED=true`)
- Response content (if `TRACE_CONTENT_ENABLED=true`)
- Token usage (prompt_tokens, completion_tokens, total_tokens)
- Latency and timing
- Provider information (OpenAI, Anthropic, etc.)
- Error details (if request fails)

## Multi-tenant Setup

Each user/team can have their own Langfuse instance:

```bash
# Team A - Uses Langfuse Cloud
curl -X POST .../pipelines -d '{
  "name": "team-a-pipeline",
  "plugins": [{
    "plugin_type": "tracing",
    "config_data": {
      "endpoint": "https://cloud.langfuse.com",
      "public_key": {"type": "literal", "value": "pk-lf-team-a-key"},
      "secret_key": {"type": "literal", "value": "sk-lf-team-a-key"}
    }
  }]
}'

# Team B - Uses Self-hosted Langfuse
curl -X POST .../pipelines -d '{
  "name": "team-b-pipeline",
  "plugins": [{
    "plugin_type": "tracing",
    "config_data": {
      "endpoint": "http://team-b-langfuse.internal:3000",
      "public_key": {"type": "literal", "value": "pk-lf-team-b-key"},
      "secret_key": {"type": "literal", "value": "sk-lf-team-b-key"}
    }
  }]
}'
```

## Troubleshooting

### Traces Not Appearing

1. **Check endpoint URL**
   - Must be base URL without `/api/public/otel/v1/traces`
   - Gateway automatically appends the path
   - Correct: `https://cloud.langfuse.com`
   - Wrong: `https://cloud.langfuse.com/api/public/otel/v1/traces`

2. **Verify API keys**
   - Public key starts with `pk-lf-`
   - Secret key starts with `sk-lf-`
   - Check they're from the correct Langfuse project

3. **Check network connectivity**
   ```bash
   docker exec hid-oauth-gateway curl -I https://cloud.langfuse.com
   ```

4. **Review gateway logs**
   ```bash
   docker logs hid-oauth-gateway 2>&1 | grep -i "opentelemetry\|langfuse\|trace"
   ```

### Authentication Errors

If you see `401 Unauthorized` in logs:
- Verify API keys are correct
- Check keys are from the same Langfuse project
- Ensure keys haven't expired or been revoked

### Self-hosted Langfuse Issues

For self-hosted Langfuse v3.22.0+:
1. Ensure OTLP endpoint is enabled (it's enabled by default)
2. Check Langfuse is accessible from gateway container
3. Verify Langfuse version supports OTLP (v3.22.0+)

## Security Best Practices

1. **Use Environment Variables**
   - Never commit API keys to version control
   - Use `type: "environment"` for secrets

2. **Kubernetes Secrets** (if using K8s)
   ```json
   {
     "public_key": {
       "type": "kubernetes",
       "secret_name": "langfuse-keys",
       "key": "public-key",
       "namespace": "default"
     },
     "secret_key": {
       "type": "kubernetes",
       "secret_name": "langfuse-keys",
       "key": "secret-key",
       "namespace": "default"
     }
   }
   ```

3. **Rotate Keys Regularly**
   - Generate new keys in Langfuse dashboard
   - Update pipeline configuration
   - Delete old keys

## Advanced Configuration

### Disable Content Tracing

To avoid sending prompt/response content to Langfuse:

```bash
export TRACE_CONTENT_ENABLED=false
```

This will only send metadata (model, tokens, latency) without actual content.

### Multiple OTLP Backends

You can send traces to multiple backends by creating separate pipelines or using OpenTelemetry Collector as a proxy.

## Support

For issues specific to:
- **Gateway integration**: Check gateway logs and this documentation
- **Langfuse platform**: Visit https://langfuse.com/docs or https://github.com/langfuse/langfuse
- **OTLP specification**: See https://opentelemetry.io/docs/specs/otlp/
