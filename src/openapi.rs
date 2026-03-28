use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::management::{
    api::routes::{
        oauth_service_routes::*,
        provider_routes::*,
        model_definition_routes::*,
        pipeline_routes::*,
    },
    dto::{
        CreateOAuthServiceRequest, UpdateOAuthServiceRequest, OAuthServiceCreatedResponse,
        OAuthServiceResponse, RotateApiKeyResponse, RegenerateTokenResponse,
        CreateProviderRequest, UpdateProviderRequest, ProviderResponse,
        CreateModelDefinitionRequest, UpdateModelDefinitionRequest, ModelDefinitionResponse,
        CreatePipelineRequestDto, UpdatePipelineRequestDto, PipelineResponseDto,
        PipelinePluginConfigDto, PluginType, SecretObject,
    },
    errors::ApiError,
};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatCompletionChoice {
    pub index: i32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatCompletionUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: Option<ChatCompletionUsage>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EmbeddingData {
    pub object: String,
    pub embedding: Vec<f32>,
    pub index: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EmbeddingResponse {
    pub object: String,
    pub data: Vec<EmbeddingData>,
    pub model: String,
    pub usage: ChatCompletionUsage,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = String)
    ),
    tag = "System"
)]
pub async fn health_handler() -> &'static str {
    "Management API is healthy"
}

#[utoipa::path(
    get,
    path = "/metrics",
    responses(
        (status = 200, description = "Prometheus metrics", body = String)
    ),
    tag = "System"
)]
pub async fn metrics_handler() -> &'static str {
    "Metrics endpoint"
}

#[utoipa::path(
    post,
    path = "/api/v1/chat/completions",
    request_body = ChatCompletionRequest,
    responses(
        (status = 200, description = "Chat completion response", body = ChatCompletionResponse),
        (status = 401, description = "Unauthorized - Invalid or missing Bearer token"),
        (status = 429, description = "Rate limit exceeded")
    ),
    tag = "LLM Gateway"
)]
pub async fn chat_completions_handler() {}

#[utoipa::path(
    post,
    path = "/api/v1/completions",
    request_body = ChatCompletionRequest,
    responses(
        (status = 200, description = "Text completion response", body = ChatCompletionResponse),
        (status = 401, description = "Unauthorized - Invalid or missing Bearer token"),
        (status = 429, description = "Rate limit exceeded")
    ),
    tag = "LLM Gateway"
)]
pub async fn completions_handler() {}

#[utoipa::path(
    post,
    path = "/api/v1/embeddings",
    request_body = EmbeddingRequest,
    responses(
        (status = 200, description = "Embedding response", body = EmbeddingResponse),
        (status = 401, description = "Unauthorized - Invalid or missing Bearer token"),
        (status = 429, description = "Rate limit exceeded")
    ),
    tag = "LLM Gateway"
)]
pub async fn embeddings_handler() {}

#[derive(OpenApi)]
#[openapi(
    paths(
        health_handler,
        metrics_handler,
        chat_completions_handler,
        completions_handler,
        embeddings_handler,
        create_oauth_service_handler,
        list_oauth_services_handler,
        get_oauth_service_handler,
        update_oauth_service_handler,
        delete_oauth_service_handler,
        rotate_api_key_handler,
        regenerate_token_handler,
        lookup_service_handler,
        create_provider_handler,
        list_providers_handler,
        get_provider_handler,
        update_provider_handler,
        delete_provider_handler,
        create_model_definition_handler,
        list_model_definitions_handler,
        get_model_definition_handler,
        get_model_definition_by_key_handler,
        update_model_definition_handler,
        delete_model_definition_handler,
        create_pipeline_handler,
        list_pipelines_handler,
        get_pipeline_handler,
        get_pipeline_by_name_handler,
        update_pipeline_handler,
        delete_pipeline_handler,
    ),
    components(
        schemas(
            ApiError,
            ChatMessage,
            ChatCompletionRequest,
            ChatCompletionChoice,
            ChatCompletionUsage,
            ChatCompletionResponse,
            EmbeddingRequest,
            EmbeddingData,
            EmbeddingResponse,
            CreateOAuthServiceRequest,
            UpdateOAuthServiceRequest,
            OAuthServiceCreatedResponse,
            OAuthServiceResponse,
            RotateApiKeyResponse,
            RegenerateTokenResponse,
            CreateProviderRequest,
            UpdateProviderRequest,
            ProviderResponse,
            CreateModelDefinitionRequest,
            UpdateModelDefinitionRequest,
            ModelDefinitionResponse,
            CreatePipelineRequestDto,
            UpdatePipelineRequestDto,
            PipelineResponseDto,
            PipelinePluginConfigDto,
            PluginType,
            SecretObject,
        )
    ),
    tags(
        (name = "LLM Gateway", description = "LLM inference endpoints (Port 3000). Requires Bearer token authentication."),
        (name = "System", description = "System health and metrics endpoints"),
        (name = "OAuth Services", description = "OAuth service management for API authentication"),
        (name = "Providers", description = "LLM provider configuration management"),
        (name = "Model Definitions", description = "Model definition management"),
        (name = "Pipelines", description = "Pipeline configuration management"),
    ),
    info(
        title = "Hid-OAuth Gateway API",
        version = "1.0.0",
        description = "HID OAuth Gateway API documentation.\n\n**LLM Gateway (Port 3000)**: Proxies requests to LLM providers. Requires Bearer token.\n\n**Management API (Port 8080)**: Admin operations for providers, models, pipelines, OAuth services.",
        contact(
            name = "AILA AI",
            url = "https://aila.ma",
            email = "ai@aila.ma"
        ),
        license(
            name = "Apache 2.0",
            url = "https://www.apache.org/licenses/LICENSE-2.0"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "LLM Gateway (requires Bearer token)"),
        (url = "http://localhost:8080", description = "Management API")
    )
)]
pub struct HidOAuthApiDoc;

pub fn get_openapi_spec() -> utoipa::openapi::OpenApi {
    HidOAuthApiDoc::openapi()
}
