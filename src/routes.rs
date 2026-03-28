use crate::state::AppState;
use axum::{
    Json, Router, body::Body, extract::Request, http::StatusCode, response::Response, routing::get,
    routing::post,
};
use axum_prometheus::PrometheusMetricLayerBuilder;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Service, ServiceExt};
use tracing::{debug, warn};

use sqlx::PgPool;

pub fn create_router(state: Arc<AppState>) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_ignore_patterns(&["/metrics", "/health"])
        .with_prefix("hid_oauth")
        .with_default_metrics()
        .build_pair();

    let dynamic_service = DynamicPipelineService::new(state.clone());

    Router::new()
        .nest_service("/api/v1", dynamic_service)
        .route("/health", get(|| async { "Working!" }))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .route(
            "/api-docs/openapi.json",
            get(|| async { Json(crate::openapi::get_openapi_spec()) }),
        )
        .layer(prometheus_layer)
        .with_state(state)
}

#[derive(Clone)]
pub struct DynamicPipelineService {
    state: Arc<AppState>,
}

impl DynamicPipelineService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }
}

impl Service<Request<Body>> for DynamicPipelineService {
    type Response = Response;
    type Error = std::convert::Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let state = self.state.clone();

        Box::pin(async move {
            let current_router = create_dynamic_pipeline_router(state);

            match current_router.oneshot(request).await {
                Ok(response) => Ok(response),
                Err(_) => {
                    let response = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::empty())
                        .unwrap();
                    Ok(response)
                }
            }
        })
    }
}

pub fn create_dynamic_pipeline_router(state: Arc<AppState>) -> Router {
    debug!("Using current pipeline router");
    Arc::try_unwrap(state.get_current_router()).unwrap_or_else(|arc_router| (*arc_router).clone())
}

pub fn create_no_config_router() -> Router {
    debug!("Creating no-config fallback router");
    Router::new()
        .route("/chat/completions", post(no_config_handler))
        .route("/completions", post(no_config_handler))
        .route("/embeddings", post(no_config_handler))
        .fallback(no_config_handler)
}

async fn no_config_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    warn!("No configuration available - returning 404 Not Found");
    Err(StatusCode::NOT_FOUND)
}

pub fn create_router_with_auth(state: Arc<AppState>, pool: PgPool) -> Router {
    use axum::middleware;
    use crate::middleware::auth::auth_middleware;

    let require_auth = std::env::var("REQUIRE_AUTH")
        .map(|v| v.to_lowercase() == "true" || v == "1")
        .unwrap_or(false);

    let (prometheus_layer, metric_handle) = PrometheusMetricLayerBuilder::new()
        .with_ignore_patterns(&["/metrics", "/health"])
        .with_prefix("hid_oauth")
        .with_default_metrics()
        .build_pair();

    let dynamic_service = DynamicPipelineService::new(state.clone());
    let pool_arc = Arc::new(pool);

    let api_router = if require_auth {
        debug!("Creating router with OAuth authentication enabled");
        let pool_for_middleware = pool_arc.clone();
        Router::new()
            .fallback_service(dynamic_service)
            .layer(middleware::from_fn(move |headers, request, next| {
                let pool = pool_for_middleware.clone();
                auth_middleware(pool, headers, request, next)
            }))
    } else {
        debug!("Creating router without authentication (REQUIRE_AUTH not set)");
        Router::new().fallback_service(dynamic_service)
    };

    Router::new()
        .nest("/api/v1", api_router)
        .route("/health", get(|| async { "Working!" }))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .route(
            "/api-docs/openapi.json",
            get(|| async { Json(crate::openapi::get_openapi_spec()) }),
        )
        .layer(prometheus_layer)
        .with_state(state)
}
