use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use utoipa::ToSchema;

pub use crate::types::ProviderType;

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum SecretObject {
    #[serde(rename = "literal")]
    Literal {
        value: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        encrypted: Option<bool>,
    },

    #[serde(rename = "kubernetes")]
    Kubernetes {
        secret_name: String,
        key: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        namespace: Option<String>,
    },

    #[serde(rename = "environment")]
    Environment { variable_name: String },
}

impl SecretObject {
    pub fn literal(value: String) -> Self {
        Self::Literal {
            value,
            encrypted: None,
        }
    }

    pub fn kubernetes(secret_name: String, key: String, namespace: Option<String>) -> Self {
        Self::Kubernetes {
            secret_name,
            key,
            namespace,
        }
    }

    pub fn environment(variable_name: String) -> Self {
        Self::Environment { variable_name }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq, Eq)]
pub struct OpenAIProviderConfig {
    pub api_key: SecretObject,
    pub organization_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq, Eq)]
pub struct AnthropicProviderConfig {
    pub api_key: SecretObject,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq, Eq)]
pub struct AzureProviderConfig {
    pub api_key: SecretObject,
    pub resource_name: String,
    pub api_version: String,
    pub base_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq, Eq)]
pub struct BedrockProviderConfig {
    pub aws_access_key_id: Option<SecretObject>,
    pub aws_secret_access_key: Option<SecretObject>,
    pub aws_session_token: Option<SecretObject>,
    pub region: String,
    pub use_iam_role: Option<bool>,
    pub inference_profile_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VertexAIProviderConfig {
    pub project_id: Option<String>,
    pub location: Option<String>,
    pub credentials_path: Option<String>,
    pub api_key: Option<SecretObject>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone, PartialEq)]
#[serde(untagged)]
pub enum ProviderConfig {
    VertexAI(VertexAIProviderConfig),
    Azure(AzureProviderConfig),
    Bedrock(BedrockProviderConfig),
    OpenAI(OpenAIProviderConfig),
    Anthropic(AnthropicProviderConfig),
}

#[derive(Serialize, Debug, ToSchema)]
pub struct CreateProviderRequest {
    pub name: String,
    #[schema(value_type = String)]
    pub provider_type: ProviderType,
    pub config: ProviderConfig,
    pub enabled: Option<bool>,
}

impl<'de> serde::Deserialize<'de> for CreateProviderRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(serde::Deserialize)]
        struct CreateProviderRequestHelper {
            name: String,
            provider_type: ProviderType,
            config: serde_json::Value,
            enabled: Option<bool>,
        }

        let helper = CreateProviderRequestHelper::deserialize(deserializer)?;

        let config = match helper.provider_type {
            ProviderType::OpenAI => {
                let config: OpenAIProviderConfig =
                    serde_json::from_value(helper.config).map_err(|e| {
                        D::Error::custom(format!("Failed to deserialize OpenAI config: {e}"))
                    })?;
                ProviderConfig::OpenAI(config)
            }
            ProviderType::Azure => {
                let config: AzureProviderConfig =
                    serde_json::from_value(helper.config).map_err(|e| {
                        D::Error::custom(format!("Failed to deserialize Azure config: {e}"))
                    })?;
                ProviderConfig::Azure(config)
            }
            ProviderType::Anthropic => {
                let config: AnthropicProviderConfig = serde_json::from_value(helper.config)
                    .map_err(|e| {
                        D::Error::custom(format!("Failed to deserialize Anthropic config: {e}"))
                    })?;
                ProviderConfig::Anthropic(config)
            }
            ProviderType::Bedrock => {
                let config: BedrockProviderConfig =
                    serde_json::from_value(helper.config).map_err(|e| {
                        D::Error::custom(format!("Failed to deserialize Bedrock config: {e}"))
                    })?;
                ProviderConfig::Bedrock(config)
            }
            ProviderType::VertexAI => {
                let config: VertexAIProviderConfig = serde_json::from_value(helper.config)
                    .map_err(|e| {
                        D::Error::custom(format!("Failed to deserialize VertexAI config: {e}"))
                    })?;
                ProviderConfig::VertexAI(config)
            }
        };

        Ok(CreateProviderRequest {
            name: helper.name,
            provider_type: helper.provider_type,
            config,
            enabled: helper.enabled,
        })
    }
}

#[derive(Serialize, Debug, ToSchema)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub config: Option<ProviderConfig>,
    pub enabled: Option<bool>,
}

impl<'de> serde::Deserialize<'de> for UpdateProviderRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(serde::Deserialize)]
        struct UpdateProviderRequestHelper {
            name: Option<String>,
            config: Option<serde_json::Value>,
            enabled: Option<bool>,
        }

        let helper = UpdateProviderRequestHelper::deserialize(deserializer)?;

        let config = helper
            .config
            .map(|config_value| {
                serde_json::from_value(config_value)
                    .map_err(|e| D::Error::custom(format!("Failed to deserialize config: {e}")))
            })
            .transpose()?;

        Ok(UpdateProviderRequest {
            name: helper.name,
            config,
            enabled: helper.enabled,
        })
    }
}

#[derive(Debug, Serialize, ToSchema, PartialEq, Clone)]
pub struct ProviderResponse {
    pub id: Uuid,
    pub name: String,
    #[schema(value_type = String)]
    pub provider_type: ProviderType,
    pub config: ProviderConfig,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'de> serde::Deserialize<'de> for ProviderResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(serde::Deserialize)]
        struct ProviderResponseHelper {
            id: Uuid,
            name: String,
            provider_type: ProviderType,
            config: serde_json::Value,
            enabled: bool,
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let helper = ProviderResponseHelper::deserialize(deserializer)?;

        let config = match helper.provider_type {
            ProviderType::OpenAI => {
                let config: OpenAIProviderConfig =
                    serde_json::from_value(helper.config).map_err(|e| {
                        D::Error::custom(format!("Failed to deserialize OpenAI config: {e}"))
                    })?;
                ProviderConfig::OpenAI(config)
            }
            ProviderType::Azure => {
                let config: AzureProviderConfig =
                    serde_json::from_value(helper.config).map_err(|e| {
                        D::Error::custom(format!("Failed to deserialize Azure config: {e}"))
                    })?;
                ProviderConfig::Azure(config)
            }
            ProviderType::Anthropic => {
                let config: AnthropicProviderConfig = serde_json::from_value(helper.config)
                    .map_err(|e| {
                        D::Error::custom(format!("Failed to deserialize Anthropic config: {e}"))
                    })?;
                ProviderConfig::Anthropic(config)
            }
            ProviderType::Bedrock => {
                let config: BedrockProviderConfig =
                    serde_json::from_value(helper.config).map_err(|e| {
                        D::Error::custom(format!("Failed to deserialize Bedrock config: {e}"))
                    })?;
                ProviderConfig::Bedrock(config)
            }
            ProviderType::VertexAI => {
                let config: VertexAIProviderConfig = serde_json::from_value(helper.config)
                    .map_err(|e| {
                        D::Error::custom(format!("Failed to deserialize VertexAI config: {e}"))
                    })?;
                ProviderConfig::VertexAI(config)
            }
        };

        Ok(ProviderResponse {
            id: helper.id,
            name: helper.name,
            provider_type: helper.provider_type,
            config,
            enabled: helper.enabled,
            created_at: helper.created_at,
            updated_at: helper.updated_at,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CreateModelDefinitionRequest {
    #[schema(example = "gpt-4o-openai")]
    pub key: String,
    #[schema(example = "gpt-4o")]
    pub model_type: String,
    pub provider_id: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(example = json!({"deployment": "my-deployment-id"}))]
    pub config_details: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct UpdateModelDefinitionRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(example = "gpt-4o-openai-updated")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(example = "gpt-4o-mini")]
    pub model_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = json!({}))]
    pub config_details: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ModelDefinitionResponse {
    pub id: Uuid,
    pub key: String,
    pub model_type: String,
    pub provider: ProviderResponse,
    pub config_details: serde_json::Value,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct ModelRouterModelEntryDto {
    #[schema(example = "gpt-4o-openai")]
    pub key: String,
    #[schema(example = 0)]
    pub priority: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ModelRouterStrategyDto {
    #[default]
    Simple,
    OrderedFallback,
    WeightedRandom,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct ModelRouterConfigDto {
    #[schema(value_type = String, example = "ordered_fallback")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<ModelRouterStrategyDto>,
    pub models: Vec<ModelRouterModelEntryDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct LoggingConfigDto {
    #[schema(value_type = String, example = "debug")]
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct TracingConfigDto {
    #[schema(value_type = String, example = "https://cloud.langfuse.com")]
    pub endpoint: String,
    #[schema(value_type = SecretObject, example = "pk-lf-...")]
    pub public_key: SecretObject,
    #[schema(value_type = SecretObject, example = "sk-lf-...")]
    pub secret_key: SecretObject,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum PluginType {
    ModelRouter,
    Logging,
    Tracing,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::ModelRouter => write!(f, "model-router"),
            PluginType::Logging => write!(f, "logging"),
            PluginType::Tracing => write!(f, "tracing"),
        }
    }
}

impl std::str::FromStr for PluginType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "model-router" => Ok(PluginType::ModelRouter),
            "logging" => Ok(PluginType::Logging),
            "tracing" => Ok(PluginType::Tracing),
            _ => Err(format!("Unknown plugin type: {s}")),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct PipelinePluginConfigDto {
    #[schema(value_type = String, example = "model-router")]
    pub plugin_type: PluginType,
    #[schema(example = json!({"strategy": "ordered_fallback", "models": [{"key": "gpt-4o", "priority": 0}]}))]
    pub config_data: serde_json::Value,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub order_in_pipeline: i32,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct CreatePipelineRequestDto {
    #[schema(example = "default_chat_pipeline")]
    pub name: String,
    #[schema(example = "chat")]
    pub pipeline_type: String,
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<PipelinePluginConfigDto>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct UpdatePipelineRequestDto {
    #[schema(example = "default_chat_pipeline_v2")]
    pub name: Option<String>,
    #[schema(example = "chat_experimental")]
    pub pipeline_type: Option<String>,
    pub description: Option<String>,
    pub plugins: Option<Vec<PipelinePluginConfigDto>>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct PipelineResponseDto {
    pub id: Uuid,
    pub name: String,
    pub pipeline_type: String,
    pub description: Option<String>,
    pub plugins: Vec<PipelinePluginConfigDto>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_logging_config_dto_serialization() {
        let config = LoggingConfigDto {
            level: "debug".to_string(),
        };

        let serialized = serde_json::to_value(&config).unwrap();
        assert_eq!(serialized, json!({"level": "debug"}));
    }

    #[test]
    fn test_logging_config_dto_deserialization() {
        let json_data = json!({"level": "info"});
        let config: LoggingConfigDto = serde_json::from_value(json_data).unwrap();
        assert_eq!(config.level, "info");
    }

    #[test]
    fn test_tracing_config_dto_serialization() {
        let config = TracingConfigDto {
            endpoint: "https://cloud.langfuse.com".to_string(),
            public_key: SecretObject::literal("pk-lf-test".to_string()),
            secret_key: SecretObject::literal("sk-lf-test".to_string()),
        };

        let serialized = serde_json::to_value(&config).unwrap();
        assert_eq!(
            serialized,
            json!({
                "endpoint": "https://cloud.langfuse.com",
                "public_key": {
                    "type": "literal",
                    "value": "pk-lf-test"
                },
                "secret_key": {
                    "type": "literal",
                    "value": "sk-lf-test"
                }
            })
        );
    }

    #[test]
    fn test_tracing_config_dto_deserialization() {
        let json_data = json!({
            "endpoint": "https://cloud.langfuse.com",
            "public_key": {
                "type": "environment",
                "variable_name": "LANGFUSE_PUBLIC_KEY"
            },
            "secret_key": {
                "type": "environment",
                "variable_name": "LANGFUSE_SECRET_KEY"
            }
        });
        let config: TracingConfigDto = serde_json::from_value(json_data).unwrap();

        assert_eq!(config.endpoint, "https://cloud.langfuse.com");
        assert_eq!(
            config.public_key,
            SecretObject::environment("LANGFUSE_PUBLIC_KEY".to_string())
        );
        assert_eq!(
            config.secret_key,
            SecretObject::environment("LANGFUSE_SECRET_KEY".to_string())
        );
    }

    #[test]
    fn test_tracing_config_dto_with_kubernetes_secret() {
        let config = TracingConfigDto {
            endpoint: "https://cloud.langfuse.com".to_string(),
            public_key: SecretObject::kubernetes(
                "langfuse-secrets".to_string(),
                "public-key".to_string(),
                Some("monitoring".to_string()),
            ),
            secret_key: SecretObject::kubernetes(
                "langfuse-secrets".to_string(),
                "secret-key".to_string(),
                Some("monitoring".to_string()),
            ),
        };

        let serialized = serde_json::to_value(&config).unwrap();
        let deserialized: TracingConfigDto = serde_json::from_value(serialized).unwrap();

        assert_eq!(deserialized.endpoint, config.endpoint);
        assert_eq!(deserialized.public_key, config.public_key);
        assert_eq!(deserialized.secret_key, config.secret_key);
    }

    #[test]
    fn test_pipeline_plugin_config_dto_with_logging() {
        let plugin_config = PipelinePluginConfigDto {
            plugin_type: PluginType::Logging,
            config_data: json!({"level": "error"}),
            enabled: true,
            order_in_pipeline: 1,
        };

        let serialized = serde_json::to_value(&plugin_config).unwrap();
        let deserialized: PipelinePluginConfigDto = serde_json::from_value(serialized).unwrap();

        assert_eq!(deserialized.plugin_type, PluginType::Logging);
        assert_eq!(deserialized.config_data, json!({"level": "error"}));
        assert!(deserialized.enabled);
        assert_eq!(deserialized.order_in_pipeline, 1);
    }

    #[test]
    fn test_pipeline_plugin_config_dto_with_tracing() {
        let plugin_config = PipelinePluginConfigDto {
            plugin_type: PluginType::Tracing,
            config_data: json!({
                "endpoint": "http://trace.example.com/v1/traces",
                "api_key": {
                    "type": "literal",
                    "value": "secret-key"
                }
            }),
            enabled: true,
            order_in_pipeline: 2,
        };

        let serialized = serde_json::to_value(&plugin_config).unwrap();
        let deserialized: PipelinePluginConfigDto = serde_json::from_value(serialized).unwrap();

        assert_eq!(deserialized.plugin_type, PluginType::Tracing);
        assert!(deserialized.enabled);
        assert_eq!(deserialized.order_in_pipeline, 2);

        let tracing_config: TracingConfigDto =
            serde_json::from_value(deserialized.config_data).unwrap();
        assert_eq!(
            tracing_config.endpoint,
            "http://trace.example.com/v1/traces"
        );
        assert_eq!(
            tracing_config.api_key,
            SecretObject::literal("secret-key".to_string())
        );
    }

    #[test]
    fn test_create_pipeline_request_with_logging_and_tracing() {
        let request = CreatePipelineRequestDto {
            name: "test-pipeline".to_string(),
            pipeline_type: "chat".to_string(),
            description: Some("Test pipeline with logging and tracing".to_string()),
            plugins: vec![
                PipelinePluginConfigDto {
                    plugin_type: PluginType::Logging,
                    config_data: json!({"level": "debug"}),
                    enabled: true,
                    order_in_pipeline: 1,
                },
                PipelinePluginConfigDto {
                    plugin_type: PluginType::Tracing,
                    config_data: json!({
                        "endpoint": "http://trace.example.com/v1/traces",
                        "api_key": {
                            "type": "environment",
                            "variable_name": "TRACE_API_KEY"
                        }
                    }),
                    enabled: true,
                    order_in_pipeline: 2,
                },
            ],
            enabled: true,
        };

        let serialized = serde_json::to_value(&request).unwrap();
        let deserialized: CreatePipelineRequestDto = serde_json::from_value(serialized).unwrap();

        assert_eq!(deserialized.name, "test-pipeline");
        assert_eq!(deserialized.plugins.len(), 2);

        let logging_plugin = &deserialized.plugins[0];
        assert_eq!(logging_plugin.plugin_type, PluginType::Logging);
        let logging_config: LoggingConfigDto =
            serde_json::from_value(logging_plugin.config_data.clone()).unwrap();
        assert_eq!(logging_config.level, "debug");

        let tracing_plugin = &deserialized.plugins[1];
        assert_eq!(tracing_plugin.plugin_type, PluginType::Tracing);
        let tracing_config: TracingConfigDto =
            serde_json::from_value(tracing_plugin.config_data.clone()).unwrap();
        assert_eq!(
            tracing_config.endpoint,
            "https://cloud.langfuse.com"
        );
        assert_eq!(
            tracing_config.public_key,
            SecretObject::environment("LANGFUSE_PUBLIC_KEY".to_string())
        );
        assert_eq!(
            tracing_config.secret_key,
            SecretObject::environment("LANGFUSE_SECRET_KEY".to_string())
        );
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateOAuthServiceRequest {
    #[schema(example = "OCR-Invoice-Service")]
    pub application_service: String,
    #[serde(default)]
    #[schema(example = 100)]
    pub rate_limit_per_minute: Option<i32>,
    #[serde(default)]
    pub allowed_models: Option<Vec<String>>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    #[serde(default)]
    #[schema(example = 24)]
    pub token_expiration_hours: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateOAuthServiceRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application_service: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit_per_minute: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_models: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schema(example = 24)]
    pub token_expiration_hours: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OAuthServiceCreatedResponse {
    pub id: Uuid,
    pub application_service: String,
    pub application_id: Uuid,
    pub api_key: String,
    pub api_secret: String,
    pub rate_limit_per_minute: i32,
    pub allowed_models: Vec<String>,
    pub token_expiration_hours: i32,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct OAuthServiceResponse {
    pub id: Uuid,
    pub application_service: String,
    pub application_id: Uuid,
    pub api_key_masked: String,
    pub rate_limit_per_minute: i32,
    pub allowed_models: Vec<String>,
    pub metadata: serde_json::Value,
    pub is_active: bool,
    pub token_expiration_hours: i32,
    pub expires_at: DateTime<Utc>,
    pub is_expired: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RotateApiKeyResponse {
    pub new_api_key: String,
    pub expires_at: DateTime<Utc>,
    pub rotated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RegenerateTokenResponse {
    pub new_api_key: String,
    pub expires_at: DateTime<Utc>,
    pub regenerated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ServiceUsageStatsResponse {
    pub oauth_service_id: Uuid,
    pub total_requests: i64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}
