use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::management::services::oauth_client_service::OAuthClientService;

#[derive(Debug, Clone)]
pub struct AuthenticatedClient {
    pub id: Uuid,
    pub application_service: String,
    pub application_id: Uuid,
    pub rate_limit_per_minute: i32,
    pub allowed_models: Vec<String>,
}

#[derive(Serialize)]
struct AuthErrorResponse {
    error: AuthErrorDetail,
}

#[derive(Serialize)]
struct AuthErrorDetail {
    message: String,
    r#type: String,
    code: String,
}

impl AuthErrorResponse {
    fn missing_api_key() -> Self {
        Self {
            error: AuthErrorDetail {
                message: "Authorization header is required".to_string(),
                r#type: "authentication_error".to_string(),
                code: "missing_api_key".to_string(),
            },
        }
    }

    fn invalid_api_key() -> Self {
        Self {
            error: AuthErrorDetail {
                message: "Invalid API key provided".to_string(),
                r#type: "authentication_error".to_string(),
                code: "invalid_api_key".to_string(),
            },
        }
    }

    fn expired_api_key() -> Self {
        Self {
            error: AuthErrorDetail {
                message: "API key has expired. Please regenerate your token using POST /api/v1/management/oauth-services/{id}/regenerate-token".to_string(),
                r#type: "authentication_error".to_string(),
                code: "expired_api_key".to_string(),
            },
        }
    }

    fn inactive_client() -> Self {
        Self {
            error: AuthErrorDetail {
                message: "Client account is inactive".to_string(),
                r#type: "permission_error".to_string(),
                code: "inactive_client".to_string(),
            },
        }
    }

    fn rate_limit_exceeded(limit: i32) -> Self {
        Self {
            error: AuthErrorDetail {
                message: format!("Rate limit exceeded. Limit: {} req/min", limit),
                r#type: "rate_limit_error".to_string(),
                code: "rate_limit_exceeded".to_string(),
            },
        }
    }
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
}

pub async fn auth_middleware(
    pool: Arc<PgPool>,
    headers: HeaderMap,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, Response> {
    let api_key = match extract_bearer_token(&headers) {
        Some(token) => token,
        None => {
            warn!("Missing Authorization header");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse::missing_api_key()),
            )
                .into_response());
        }
    };

    let oauth_service = OAuthClientService::new((*pool).clone());

    let client = match oauth_service.validate_api_key(api_key).await {
        Ok(client) => client,
        Err(e) => {
            let error_msg = format!("{:?}", e);
            if error_msg.contains("expired") {
                warn!("Expired API key attempted");
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(AuthErrorResponse::expired_api_key()),
                )
                    .into_response());
            }
            warn!("Invalid API key attempted");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthErrorResponse::invalid_api_key()),
            )
                .into_response());
        }
    };

    if !client.is_active {
        warn!("Inactive client attempted access: {}", client.application_service);
        return Err((
            StatusCode::FORBIDDEN,
            Json(AuthErrorResponse::inactive_client()),
        )
            .into_response());
    }

    match oauth_service
        .check_rate_limit(client.id, client.rate_limit_per_minute)
        .await
    {
        Ok(within_limit) => {
            if !within_limit {
                warn!(
                    "Rate limit exceeded for client: {} (limit: {})",
                    client.application_service, client.rate_limit_per_minute
                );
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(AuthErrorResponse::rate_limit_exceeded(
                        client.rate_limit_per_minute,
                    )),
                )
                    .into_response());
            }
        }
        Err(e) => {
            warn!("Failed to check rate limit: {:?}", e);
        }
    }

    let allowed_models: Vec<String> =
        serde_json::from_value(client.allowed_models.clone()).unwrap_or_default();

    let auth_client = AuthenticatedClient {
        id: client.id,
        application_service: client.application_service.clone(),
        application_id: client.application_id,
        rate_limit_per_minute: client.rate_limit_per_minute,
        allowed_models,
    };

    debug!(
        "Request authenticated for client: {} ({})",
        auth_client.application_service, auth_client.application_id
    );

    request.extensions_mut().insert(auth_client);

    Ok(next.run(request).await)
}

pub async fn optional_auth_middleware(
    pool: Arc<PgPool>,
    headers: HeaderMap,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    if let Some(api_key) = extract_bearer_token(&headers) {
        let oauth_service = OAuthClientService::new((*pool).clone());

        if let Ok(client) = oauth_service.validate_api_key(api_key).await {
            if client.is_active {
                let allowed_models: Vec<String> =
                    serde_json::from_value(client.allowed_models.clone()).unwrap_or_default();

                let auth_client = AuthenticatedClient {
                    id: client.id,
                    application_service: client.application_service.clone(),
                    application_id: client.application_id,
                    rate_limit_per_minute: client.rate_limit_per_minute,
                    allowed_models,
                };

                debug!(
                    "Request optionally authenticated for client: {}",
                    auth_client.application_service
                );
                request.extensions_mut().insert(auth_client);
            }
        }
    }

    next.run(request).await
}
