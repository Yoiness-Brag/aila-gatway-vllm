use serde_json::Value as JsonValue;
use sqlx::{PgPool, Result, query, query_as, types::Uuid};

use crate::management::db::models::OAuthClient;

#[derive(Debug, Clone)]
pub struct OAuthClientRepository {
    pool: PgPool,
}

impl OAuthClientRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        application_service: &str,
        application_id: Uuid,
        api_key_hash: &str,
        api_secret_hash: &str,
        rate_limit_per_minute: i32,
        allowed_models: JsonValue,
        metadata: JsonValue,
        token_expiration_hours: i32,
    ) -> Result<OAuthClient> {
        let new_id = Uuid::new_v4();
        query_as!(
            OAuthClient,
            r#"
            INSERT INTO hub_llmgateway_oauth_clients 
            (id, application_service, application_id, api_key_hash, api_secret_hash, rate_limit_per_minute, allowed_models, metadata, is_active, token_expiration_hours, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, $9, NOW() + make_interval(hours => $9))
            RETURNING id, application_service, application_id, api_key_hash, api_secret_hash, rate_limit_per_minute, 
                      allowed_models, metadata, is_active, token_expiration_hours, expires_at, created_at, updated_at, last_used_at
            "#,
            new_id,
            application_service,
            application_id,
            api_key_hash,
            api_secret_hash,
            rate_limit_per_minute,
            allowed_models,
            metadata,
            token_expiration_hours
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<OAuthClient>> {
        query_as!(
            OAuthClient,
            r#"
            SELECT id, application_service, application_id, api_key_hash, api_secret_hash, rate_limit_per_minute,
                   allowed_models, metadata, is_active, token_expiration_hours, expires_at, created_at, updated_at, last_used_at
            FROM hub_llmgateway_oauth_clients
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_application_service(&self, application_service: &str) -> Result<Option<OAuthClient>> {
        query_as!(
            OAuthClient,
            r#"
            SELECT id, application_service, application_id, api_key_hash, api_secret_hash, rate_limit_per_minute,
                   allowed_models, metadata, is_active, token_expiration_hours, expires_at, created_at, updated_at, last_used_at
            FROM hub_llmgateway_oauth_clients
            WHERE application_service = $1
            "#,
            application_service
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_application_id(&self, application_id: Uuid) -> Result<Option<OAuthClient>> {
        query_as!(
            OAuthClient,
            r#"
            SELECT id, application_service, application_id, api_key_hash, api_secret_hash, rate_limit_per_minute,
                   allowed_models, metadata, is_active, token_expiration_hours, expires_at, created_at, updated_at, last_used_at
            FROM hub_llmgateway_oauth_clients
            WHERE application_id = $1
            "#,
            application_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_api_key_hash(&self, api_key_hash: &str) -> Result<Option<OAuthClient>> {
        query_as!(
            OAuthClient,
            r#"
            SELECT id, application_service, application_id, api_key_hash, api_secret_hash, rate_limit_per_minute,
                   allowed_models, metadata, is_active, token_expiration_hours, expires_at, created_at, updated_at, last_used_at
            FROM hub_llmgateway_oauth_clients
            WHERE api_key_hash = $1 AND is_active = true AND expires_at > NOW()
            "#,
            api_key_hash
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list(&self) -> Result<Vec<OAuthClient>> {
        query_as!(
            OAuthClient,
            r#"
            SELECT id, application_service, application_id, api_key_hash, api_secret_hash, rate_limit_per_minute,
                   allowed_models, metadata, is_active, token_expiration_hours, expires_at, created_at, updated_at, last_used_at
            FROM hub_llmgateway_oauth_clients
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        application_service: Option<&str>,
        rate_limit_per_minute: Option<i32>,
        allowed_models: Option<JsonValue>,
        metadata: Option<JsonValue>,
        is_active: Option<bool>,
        token_expiration_hours: Option<i32>,
    ) -> Result<Option<OAuthClient>> {
        let current = self.find_by_id(id).await?.ok_or(sqlx::Error::RowNotFound)?;

        let app_service = application_service.unwrap_or(&current.application_service);
        let rate_limit = rate_limit_per_minute.unwrap_or(current.rate_limit_per_minute);
        let models = allowed_models.unwrap_or(current.allowed_models.clone());
        let meta = metadata.unwrap_or(current.metadata.clone());
        let active = is_active.unwrap_or(current.is_active);
        let expiration_hours = token_expiration_hours.unwrap_or(current.token_expiration_hours);

        query_as!(
            OAuthClient,
            r#"
            UPDATE hub_llmgateway_oauth_clients
            SET application_service = $1, rate_limit_per_minute = $2, allowed_models = $3, 
                metadata = $4, is_active = $5, token_expiration_hours = $6, updated_at = NOW()
            WHERE id = $7
            RETURNING id, application_service, application_id, api_key_hash, api_secret_hash, rate_limit_per_minute,
                      allowed_models, metadata, is_active, token_expiration_hours, expires_at, created_at, updated_at, last_used_at
            "#,
            app_service,
            rate_limit,
            models,
            meta,
            active,
            expiration_hours,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_api_key_hash(
        &self,
        id: Uuid,
        new_api_key_hash: &str,
    ) -> Result<Option<OAuthClient>> {
        query_as!(
            OAuthClient,
            r#"
            UPDATE hub_llmgateway_oauth_clients
            SET api_key_hash = $1, expires_at = NOW() + make_interval(hours => token_expiration_hours), updated_at = NOW()
            WHERE id = $2
            RETURNING id, application_service, application_id, api_key_hash, api_secret_hash, rate_limit_per_minute,
                      allowed_models, metadata, is_active, token_expiration_hours, expires_at, created_at, updated_at, last_used_at
            "#,
            new_api_key_hash,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn regenerate_token(&self, id: Uuid, new_api_key_hash: &str) -> Result<Option<OAuthClient>> {
        query_as!(
            OAuthClient,
            r#"
            UPDATE hub_llmgateway_oauth_clients
            SET api_key_hash = $1, expires_at = NOW() + make_interval(hours => token_expiration_hours), updated_at = NOW()
            WHERE id = $2
            RETURNING id, application_service, application_id, api_key_hash, api_secret_hash, rate_limit_per_minute,
                      allowed_models, metadata, is_active, token_expiration_hours, expires_at, created_at, updated_at, last_used_at
            "#,
            new_api_key_hash,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn update_last_used(&self, id: Uuid) -> Result<()> {
        query!(
            r#"
            UPDATE hub_llmgateway_oauth_clients
            SET last_used_at = NOW()
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<u64> {
        let result = query!(
            r#"
            DELETE FROM hub_llmgateway_oauth_clients
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn count_requests_in_window(&self, oauth_service_id: Uuid, window_seconds: i64) -> Result<i64> {
        let count: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM hub_llmgateway_client_usage_logs
            WHERE oauth_service_id = $1 AND request_timestamp > NOW() - make_interval(secs => $2)
            "#
        )
        .bind(oauth_service_id)
        .bind(window_seconds as f64)
        .fetch_one(&self.pool)
        .await?;
        Ok(count.unwrap_or(0))
    }

    pub async fn log_usage(
        &self,
        oauth_service_id: Uuid,
        endpoint: &str,
        model: Option<&str>,
        tokens_used: Option<i32>,
        latency_ms: i32,
        response_status: i32,
        error_message: Option<&str>,
    ) -> Result<()> {
        let log_id = Uuid::new_v4();
        query!(
            r#"
            INSERT INTO hub_llmgateway_client_usage_logs 
            (id, oauth_service_id, endpoint, model, tokens_used, latency_ms, response_status, error_message)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            log_id,
            oauth_service_id,
            endpoint,
            model,
            tokens_used,
            latency_ms,
            response_status,
            error_message
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
