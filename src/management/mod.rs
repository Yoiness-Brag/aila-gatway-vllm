pub mod api;
pub mod db;
pub mod dto;
pub mod errors;
pub mod services;
pub mod state;

pub use state::{DbBasedConfigIntegration, db_based_config_integration};

use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use self::db::repositories::{
    model_definition_repository::ModelDefinitionRepository,
    pipeline_repository::PipelineRepository,
};

use self::services::{
    config_provider_service::ConfigProviderService,
    model_definition_service::ModelDefinitionService,
    oauth_client_service::OAuthClientService,
    pipeline_service::PipelineService,
    provider_service::ProviderService,
};

use crate::openapi::HidOAuthApiDoc;

#[derive(Clone, axum::extract::FromRef)]
pub struct AppState {
    pub db_pool: PgPool,
    pub provider_service: Arc<ProviderService>,
    pub model_definition_service: Arc<ModelDefinitionService>,
    pub pipeline_service: Arc<PipelineService>,
    pub config_provider_service: Arc<ConfigProviderService>,
    pub oauth_client_service: Arc<OAuthClientService>,
}

pub fn management_api_bundle(pool: PgPool) -> (Router, Arc<ConfigProviderService>) {
    let model_definition_repo_for_pipeline_service =
        Arc::new(ModelDefinitionRepository::new(pool.clone()));
    let pipeline_repo_for_pipeline_service = Arc::new(PipelineRepository::new(pool.clone()));

    let provider_service = Arc::new(ProviderService::new(pool.clone()));
    let model_definition_service = Arc::new(ModelDefinitionService::new(pool.clone()));
    let pipeline_service = Arc::new(PipelineService::new(
        pipeline_repo_for_pipeline_service,
        model_definition_repo_for_pipeline_service,
    ));

    let config_provider_service = Arc::new(ConfigProviderService::new(
        provider_service.clone(),
        model_definition_service.clone(),
        pipeline_service.clone(),
    ));

    let oauth_client_service = Arc::new(OAuthClientService::new(pool.clone()));

    let app_state = AppState {
        db_pool: pool.clone(),
        provider_service,
        model_definition_service,
        pipeline_service,
        config_provider_service: config_provider_service.clone(),
        oauth_client_service,
    };

    let router = Router::new()
        .nest(
            "/api/v1/management/providers",
            api::routes::provider_routes::provider_routes(),
        )
        .nest(
            "/api/v1/management/model-definitions",
            api::routes::model_definition_routes::model_definition_routes(),
        )
        .nest(
            "/api/v1/management/pipelines",
            api::routes::pipeline_routes::pipeline_routes(),
        )
        .nest(
            "/api/v1/management/oauth-services",
            api::routes::oauth_service_routes::oauth_service_routes(),
        )
        .route(
            "/health",
            axum::routing::get(|| async { "Management API is healthy" }),
        )
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", HidOAuthApiDoc::openapi()))
        .with_state(app_state);

    (router, config_provider_service)
}
