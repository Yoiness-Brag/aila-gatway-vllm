use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{
    FromRow,
    types::{JsonValue, Uuid},
};

#[derive(Debug, sqlx::FromRow)]
pub struct Provider {
    pub id: Uuid,
    pub name: String,
    pub provider_type: String, // Stored as VARCHAR in DB, maps to ProviderType enum conceptually
    pub config_details: JsonValue, // Stored as JSONB in DB
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Clone)] // Added Clone here for potential use in services
pub struct ModelDefinition {
    pub id: Uuid,
    pub key: String,
    pub model_type: String,
    pub provider_id: Uuid,
    pub config_details: Option<serde_json::Value>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Clone)]
pub struct Pipeline {
    pub id: Uuid,
    pub name: String,
    pub pipeline_type: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Clone)]
pub struct PipelinePluginConfig {
    pub id: Uuid,
    pub pipeline_id: Uuid,
    pub plugin_type: String,
    pub config_data: serde_json::Value, // Stored as JSONB
    pub enabled: bool,
    pub order_in_pipeline: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PipelineWithPlugins {
    pub id: Uuid,
    pub name: String,
    pub pipeline_type: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub plugins: Vec<PipelinePluginConfig>,
}

#[derive(Debug, FromRow, Clone)]
pub struct OAuthClient {
    pub id: Uuid,
    pub application_service: String,
    pub application_id: Uuid,
    pub api_key_hash: String,
    pub api_secret_hash: String,
    pub rate_limit_per_minute: i32,
    pub allowed_models: serde_json::Value,
    pub metadata: serde_json::Value,
    pub is_active: bool,
    pub token_expiration_hours: i32,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow, Clone)]
pub struct ClientUsageLog {
    pub id: Uuid,
    pub oauth_service_id: Uuid,
    pub endpoint: String,
    pub model: Option<String>,
    pub tokens_used: Option<i32>,
    pub latency_ms: i32,
    pub response_status: i32,
    pub error_message: Option<String>,
    pub request_timestamp: DateTime<Utc>,
}
