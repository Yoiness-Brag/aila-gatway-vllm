use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
};
use chrono::Utc;
use uuid::Uuid;

use crate::management::{
    AppState,
    dto::{
        CreateOAuthServiceRequest, UpdateOAuthServiceRequest, OAuthServiceCreatedResponse,
        OAuthServiceResponse, RotateApiKeyResponse, RegenerateTokenResponse,
    },
    errors::ApiError,
};

pub fn oauth_service_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            post(create_oauth_service_handler).get(list_oauth_services_handler),
        )
        .route(
            "/{id}",
            get(get_oauth_service_handler)
                .put(update_oauth_service_handler)
                .delete(delete_oauth_service_handler),
        )
        .route("/{id}/rotate-key", post(rotate_api_key_handler))
        .route("/{id}/regenerate-token", post(regenerate_token_handler))
        .route("/lookup", get(lookup_service_handler))
}

#[utoipa::path(
    post,
    path = "/api/v1/management/oauth-services",
    request_body = CreateOAuthServiceRequest,
    responses(
        (status = 201, description = "OAuth service created successfully", body = OAuthServiceCreatedResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 409, description = "Conflict - service name already exists", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "OAuth Services"
)]
#[axum::debug_handler]
pub async fn create_oauth_service_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateOAuthServiceRequest>,
) -> Result<(StatusCode, Json<OAuthServiceCreatedResponse>), ApiError> {
    let service = &app_state.oauth_client_service;
    let result = service.create_oauth_client(
        payload.application_service,
        payload.rate_limit_per_minute,
        payload.allowed_models,
        payload.metadata,
        payload.token_expiration_hours,
    ).await?;

    let response = OAuthServiceCreatedResponse {
        id: result.id,
        application_service: result.application_service,
        application_id: result.application_id,
        api_key: result.api_key,
        api_secret: result.api_secret,
        rate_limit_per_minute: result.rate_limit_per_minute,
        allowed_models: result.allowed_models,
        token_expiration_hours: result.token_expiration_hours,
        expires_at: result.expires_at,
        created_at: Utc::now(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/api/v1/management/oauth-services",
    responses(
        (status = 200, description = "List of OAuth services", body = Vec<OAuthServiceResponse>),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "OAuth Services"
)]
#[axum::debug_handler]
pub async fn list_oauth_services_handler(
    State(app_state): State<AppState>,
) -> Result<Json<Vec<OAuthServiceResponse>>, ApiError> {
    let service = &app_state.oauth_client_service;
    let clients = service.list_oauth_clients().await?;
    
    let responses: Vec<OAuthServiceResponse> = clients
        .into_iter()
        .map(|c| {
            let allowed_models: Vec<String> = serde_json::from_value(c.allowed_models.clone())
                .unwrap_or_default();
            let is_expired = c.expires_at < Utc::now();
            OAuthServiceResponse {
                id: c.id,
                application_service: c.application_service,
                application_id: c.application_id,
                api_key_masked: "hid_live_***".to_string(),
                rate_limit_per_minute: c.rate_limit_per_minute,
                allowed_models,
                metadata: c.metadata,
                is_active: c.is_active,
                token_expiration_hours: c.token_expiration_hours,
                expires_at: c.expires_at,
                is_expired,
                created_at: c.created_at,
                updated_at: c.updated_at,
                last_used_at: c.last_used_at,
            }
        })
        .collect();

    Ok(Json(responses))
}

#[utoipa::path(
    get,
    path = "/api/v1/management/oauth-services/{id}",
    params(
        ("id" = Uuid, Path, description = "OAuth Service ID")
    ),
    responses(
        (status = 200, description = "OAuth service found", body = OAuthServiceResponse),
        (status = 404, description = "OAuth service not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "OAuth Services"
)]
#[axum::debug_handler]
pub async fn get_oauth_service_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<OAuthServiceResponse>, ApiError> {
    let service = &app_state.oauth_client_service;
    let client = service.get_oauth_client(id).await?;
    
    let allowed_models: Vec<String> = serde_json::from_value(client.allowed_models.clone())
        .unwrap_or_default();

    let is_expired = client.expires_at < Utc::now();
    let response = OAuthServiceResponse {
        id: client.id,
        application_service: client.application_service,
        application_id: client.application_id,
        api_key_masked: "hid_live_***".to_string(),
        rate_limit_per_minute: client.rate_limit_per_minute,
        allowed_models,
        metadata: client.metadata,
        is_active: client.is_active,
        token_expiration_hours: client.token_expiration_hours,
        expires_at: client.expires_at,
        is_expired,
        created_at: client.created_at,
        updated_at: client.updated_at,
        last_used_at: client.last_used_at,
    };

    Ok(Json(response))
}

#[utoipa::path(
    put,
    path = "/api/v1/management/oauth-services/{id}",
    request_body = UpdateOAuthServiceRequest,
    params(
        ("id" = Uuid, Path, description = "OAuth Service ID")
    ),
    responses(
        (status = 200, description = "OAuth service updated successfully", body = OAuthServiceResponse),
        (status = 400, description = "Invalid request", body = ApiError),
        (status = 404, description = "OAuth service not found", body = ApiError),
        (status = 409, description = "Conflict - service name already exists", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "OAuth Services"
)]
#[axum::debug_handler]
pub async fn update_oauth_service_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOAuthServiceRequest>,
) -> Result<Json<OAuthServiceResponse>, ApiError> {
    let service = &app_state.oauth_client_service;
    
    let metadata_json = payload.metadata;
    
    let client = service.update_oauth_client(
        id,
        payload.application_service,
        payload.rate_limit_per_minute,
        payload.allowed_models,
        metadata_json,
        payload.is_active,
        payload.token_expiration_hours,
    ).await?;

    let allowed_models: Vec<String> = serde_json::from_value(client.allowed_models.clone())
        .unwrap_or_default();

    let is_expired = client.expires_at < Utc::now();
    let response = OAuthServiceResponse {
        id: client.id,
        application_service: client.application_service,
        application_id: client.application_id,
        api_key_masked: "hid_live_***".to_string(),
        rate_limit_per_minute: client.rate_limit_per_minute,
        allowed_models,
        metadata: client.metadata,
        is_active: client.is_active,
        token_expiration_hours: client.token_expiration_hours,
        expires_at: client.expires_at,
        is_expired,
        created_at: client.created_at,
        updated_at: client.updated_at,
        last_used_at: client.last_used_at,
    };

    Ok(Json(response))
}

#[utoipa::path(
    delete,
    path = "/api/v1/management/oauth-services/{id}",
    params(
        ("id" = Uuid, Path, description = "OAuth Service ID")
    ),
    responses(
        (status = 204, description = "OAuth service deleted successfully"),
        (status = 404, description = "OAuth service not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "OAuth Services"
)]
#[axum::debug_handler]
pub async fn delete_oauth_service_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let service = &app_state.oauth_client_service;
    service.delete_oauth_client(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/management/oauth-services/{id}/rotate-key",
    params(
        ("id" = Uuid, Path, description = "OAuth Service ID")
    ),
    responses(
        (status = 200, description = "API key rotated successfully", body = RotateApiKeyResponse),
        (status = 404, description = "OAuth service not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "OAuth Services"
)]
#[axum::debug_handler]
pub async fn rotate_api_key_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RotateApiKeyResponse>, ApiError> {
    let service = &app_state.oauth_client_service;
    let new_api_key = service.rotate_api_key(id).await?;
    
    let client = service.get_oauth_client(id).await?;

    let response = RotateApiKeyResponse {
        new_api_key,
        expires_at: client.expires_at,
        rotated_at: Utc::now(),
    };

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/management/oauth-services/{id}/regenerate-token",
    params(
        ("id" = Uuid, Path, description = "OAuth Service ID")
    ),
    responses(
        (status = 200, description = "Token regenerated successfully", body = RegenerateTokenResponse),
        (status = 404, description = "OAuth service not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "OAuth Services"
)]
#[axum::debug_handler]
pub async fn regenerate_token_handler(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RegenerateTokenResponse>, ApiError> {
    let service = &app_state.oauth_client_service;
    let (new_api_key, expires_at) = service.regenerate_token(id).await?;

    let response = RegenerateTokenResponse {
        new_api_key,
        expires_at,
        regenerated_at: Utc::now(),
    };

    Ok(Json(response))
}

#[derive(Debug, serde::Deserialize)]
pub struct LookupQuery {
    pub application_service: Option<String>,
    pub application_id: Option<Uuid>,
}

#[utoipa::path(
    get,
    path = "/api/v1/management/oauth-services/lookup",
    params(
        ("application_service" = Option<String>, Query, description = "Application service name to search for"),
        ("application_id" = Option<Uuid>, Query, description = "Application ID (UUID) to search for")
    ),
    responses(
        (status = 200, description = "OAuth service found", body = OAuthServiceResponse),
        (status = 400, description = "Must provide application_service or application_id", body = ApiError),
        (status = 404, description = "OAuth service not found", body = ApiError),
        (status = 500, description = "Internal server error", body = ApiError)
    ),
    tag = "OAuth Services"
)]
#[axum::debug_handler]
pub async fn lookup_service_handler(
    State(app_state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<LookupQuery>,
) -> Result<Json<OAuthServiceResponse>, ApiError> {
    let service = &app_state.oauth_client_service;
    
    let client = match (&query.application_service, &query.application_id) {
        (Some(app_service), _) => {
            service.find_by_application_service(app_service).await?
        }
        (None, Some(app_id)) => {
            service.find_by_application_id(*app_id).await?
        }
        (None, None) => {
            return Err(ApiError::ValidationError("Must provide 'application_service' or 'application_id' query parameter".to_string()));
        }
    };
    
    let client = client.ok_or_else(|| ApiError::NotFound("OAuth client not found".to_string()))?;
    
    let allowed_models: Vec<String> = serde_json::from_value(client.allowed_models.clone())
        .unwrap_or_default();
    let is_expired = client.expires_at < Utc::now();
    
    let response = OAuthServiceResponse {
        id: client.id,
        application_service: client.application_service,
        application_id: client.application_id,
        api_key_masked: "hid_live_***".to_string(),
        rate_limit_per_minute: client.rate_limit_per_minute,
        allowed_models,
        metadata: client.metadata,
        is_active: client.is_active,
        token_expiration_hours: client.token_expiration_hours,
        expires_at: client.expires_at,
        is_expired,
        created_at: client.created_at,
        updated_at: client.updated_at,
        last_used_at: client.last_used_at,
    };

    Ok(Json(response))
}
