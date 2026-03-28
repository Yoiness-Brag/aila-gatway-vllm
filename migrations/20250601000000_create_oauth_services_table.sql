-- Migration: Create OAuth Services table for client authentication
-- Final schema with application_id as UUID, no client_id

-- Table for OAuth Services
CREATE TABLE hub_llmgateway_oauth_clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    application_service VARCHAR(255) NOT NULL,
    application_id UUID UNIQUE NOT NULL DEFAULT gen_random_uuid(),
    api_key_hash VARCHAR(255) NOT NULL,
    api_secret_hash VARCHAR(255) NOT NULL,
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 100,
    allowed_models JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    token_expiration_hours INTEGER NOT NULL DEFAULT 24,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '24 hours'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

-- Create indexes for fast lookups
CREATE INDEX idx_oauth_clients_application_service ON hub_llmgateway_oauth_clients(application_service);
CREATE INDEX idx_oauth_clients_application_id ON hub_llmgateway_oauth_clients(application_id);
CREATE INDEX idx_oauth_clients_api_key_hash ON hub_llmgateway_oauth_clients(api_key_hash);
CREATE INDEX idx_oauth_clients_is_active ON hub_llmgateway_oauth_clients(is_active);
CREATE INDEX idx_oauth_clients_expires_at ON hub_llmgateway_oauth_clients(expires_at);
CREATE INDEX idx_oauth_clients_created_at ON hub_llmgateway_oauth_clients(created_at DESC);

-- Trigger to update 'updated_at' timestamp on row update
CREATE TRIGGER update_oauth_clients_updated_at
BEFORE UPDATE ON hub_llmgateway_oauth_clients
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();

-- Table for Service Usage Logs
CREATE TABLE hub_llmgateway_client_usage_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    oauth_service_id UUID NOT NULL REFERENCES hub_llmgateway_oauth_clients(id) ON DELETE CASCADE,
    endpoint VARCHAR(255) NOT NULL,
    model VARCHAR(255),
    tokens_used INTEGER,
    latency_ms INTEGER NOT NULL,
    response_status INTEGER NOT NULL,
    error_message TEXT,
    request_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for usage logs
CREATE INDEX idx_client_usage_logs_oauth_service_id ON hub_llmgateway_client_usage_logs(oauth_service_id);
CREATE INDEX idx_client_usage_logs_timestamp ON hub_llmgateway_client_usage_logs(request_timestamp DESC);
CREATE INDEX idx_client_usage_logs_model ON hub_llmgateway_client_usage_logs(model);
