use serde_json::{Value as JsonValue, json};
use sqlx::{PgPool, types::Uuid};
use std::sync::Arc;

use crate::management::{
    db::repositories::oauth_client_repository::OAuthClientRepository,
    errors::ApiError,
};

const API_KEY_PREFIX: &str = "hid_live_";
const API_SECRET_PREFIX: &str = "hid_secret_";
const RANDOM_STRING_LENGTH: usize = 32;

#[derive(Debug, Clone)]
pub struct OAuthClientService {
    repo: Arc<OAuthClientRepository>,
}

#[derive(Debug, Clone)]
pub struct CreateOAuthClientResult {
    pub id: Uuid,
    pub application_service: String,
    pub application_id: Uuid,
    pub api_key: String,
    pub api_secret: String,
    pub rate_limit_per_minute: i32,
    pub allowed_models: Vec<String>,
    pub token_expiration_hours: i32,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl OAuthClientService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repo: Arc::new(OAuthClientRepository::new(pool)),
        }
    }

    fn generate_random_string(length: usize) -> String {
        use std::iter;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        
        let mut rng_bytes = [0u8; 64];
        if let Ok(()) = getrandom::getrandom(&mut rng_bytes) {
            iter::repeat(())
                .take(length)
                .enumerate()
                .map(|(i, _)| {
                    let idx = (rng_bytes[i % 64] as usize) % CHARSET.len();
                    CHARSET[idx] as char
                })
                .collect()
        } else {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            format!("{:x}", now)[..length.min(32)].to_string()
        }
    }

    fn generate_api_key() -> String {
        format!("{}{}", API_KEY_PREFIX, Self::generate_random_string(RANDOM_STRING_LENGTH))
    }

    fn generate_api_secret() -> String {
        format!("{}{}", API_SECRET_PREFIX, Self::generate_random_string(RANDOM_STRING_LENGTH))
    }

    fn generate_application_id() -> Uuid {
        Uuid::new_v4()
    }

    fn hash_secret(secret: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        secret.hash(&mut hasher);
        let hash1 = hasher.finish();
        
        let mut hasher2 = DefaultHasher::new();
        hash1.hash(&mut hasher2);
        secret.hash(&mut hasher2);
        let hash2 = hasher2.finish();
        
        format!("{:016x}{:016x}", hash1, hash2)
    }

    pub async fn create_oauth_client(
        &self,
        application_service: String,
        rate_limit_per_minute: Option<i32>,
        allowed_models: Option<Vec<String>>,
        metadata: Option<JsonValue>,
        token_expiration_hours: Option<i32>,
    ) -> Result<CreateOAuthClientResult, ApiError> {
        if let Some(_existing) = self.repo.find_by_application_service(&application_service).await? {
            return Err(ApiError::Conflict(format!(
                "OAuth client with application_service '{}' already exists.",
                application_service
            )));
        }

        let application_id = Self::generate_application_id();
        let api_key = Self::generate_api_key();
        let api_secret = Self::generate_api_secret();
        
        let api_key_hash = Self::hash_secret(&api_key);
        let api_secret_hash = Self::hash_secret(&api_secret);
        
        let rate_limit = rate_limit_per_minute.unwrap_or(100);
        let models = allowed_models.clone().unwrap_or_default();
        let models_json = json!(models);
        let meta = metadata.unwrap_or(json!({}));
        let expiration_hours = token_expiration_hours.unwrap_or(24);

        let db_client = self.repo.create(
            &application_service,
            application_id,
            &api_key_hash,
            &api_secret_hash,
            rate_limit,
            models_json,
            meta,
            expiration_hours,
        ).await?;

        Ok(CreateOAuthClientResult {
            id: db_client.id,
            application_service: db_client.application_service,
            application_id: db_client.application_id,
            api_key,
            api_secret,
            rate_limit_per_minute: db_client.rate_limit_per_minute,
            allowed_models: models,
            token_expiration_hours: db_client.token_expiration_hours,
            expires_at: db_client.expires_at,
        })
    }

    pub async fn find_by_application_service(&self, application_service: &str) -> Result<Option<crate::management::db::models::OAuthClient>, ApiError> {
        Ok(self.repo.find_by_application_service(application_service).await?)
    }

    pub async fn find_by_application_id(&self, application_id: Uuid) -> Result<Option<crate::management::db::models::OAuthClient>, ApiError> {
        Ok(self.repo.find_by_application_id(application_id).await?)
    }

    pub async fn get_oauth_client(&self, id: Uuid) -> Result<crate::management::db::models::OAuthClient, ApiError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("OAuth client with ID {id} not found.")))
    }

    pub async fn list_oauth_clients(&self) -> Result<Vec<crate::management::db::models::OAuthClient>, ApiError> {
        Ok(self.repo.list().await?)
    }

    pub async fn update_oauth_client(
        &self,
        id: Uuid,
        application_service: Option<String>,
        rate_limit_per_minute: Option<i32>,
        allowed_models: Option<Vec<String>>,
        metadata: Option<JsonValue>,
        is_active: Option<bool>,
        token_expiration_hours: Option<i32>,
    ) -> Result<crate::management::db::models::OAuthClient, ApiError> {
        self.repo.find_by_id(id).await?.ok_or_else(|| {
            ApiError::NotFound(format!("OAuth client with ID {id} not found."))
        })?;

        if let Some(ref new_service) = application_service {
            if let Some(existing) = self.repo.find_by_application_service(new_service).await? {
                if existing.id != id {
                    return Err(ApiError::Conflict(format!(
                        "Another OAuth client with application_service '{new_service}' already exists."
                    )));
                }
            }
        }

        let models_json = allowed_models.map(|m| json!(m));

        self.repo
            .update(
                id,
                application_service.as_deref(),
                rate_limit_per_minute,
                models_json,
                metadata,
                is_active,
                token_expiration_hours,
            )
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("OAuth client with ID {id} not found after update.")))
    }

    pub async fn delete_oauth_client(&self, id: Uuid) -> Result<(), ApiError> {
        let affected = self.repo.delete(id).await?;
        if affected == 0 {
            Err(ApiError::NotFound(format!(
                "OAuth client with ID {id} not found, nothing deleted."
            )))
        } else {
            Ok(())
        }
    }

    pub async fn rotate_api_key(&self, id: Uuid) -> Result<String, ApiError> {
        self.repo.find_by_id(id).await?.ok_or_else(|| {
            ApiError::NotFound(format!("OAuth client with ID {id} not found."))
        })?;

        let new_api_key = Self::generate_api_key();
        let new_api_key_hash = Self::hash_secret(&new_api_key);

        self.repo.update_api_key_hash(id, &new_api_key_hash).await?;

        Ok(new_api_key)
    }

    pub async fn validate_api_key(&self, api_key: &str) -> Result<crate::management::db::models::OAuthClient, ApiError> {
        let api_key_hash = Self::hash_secret(api_key);
        
        let client = self.repo
            .find_by_api_key_hash(&api_key_hash)
            .await?
            .ok_or_else(|| ApiError::Unauthorized("Invalid or expired API key. Please regenerate your token.".to_string()))?;

        if !client.is_active {
            return Err(ApiError::Forbidden("Client account is inactive".to_string()));
        }

        if client.expires_at < chrono::Utc::now() {
            return Err(ApiError::Unauthorized(
                "API key has expired. Please regenerate your token using POST /api/v1/management/oauth-services/{id}/regenerate-token".to_string()
            ));
        }

        let repo = self.repo.clone();
        let service_id = client.id;
        tokio::spawn(async move {
            let _ = repo.update_last_used(service_id).await;
        });

        Ok(client)
    }

    pub async fn regenerate_token(&self, id: Uuid) -> Result<(String, chrono::DateTime<chrono::Utc>), ApiError> {
        self.repo.find_by_id(id).await?.ok_or_else(|| {
            ApiError::NotFound(format!("OAuth client with ID {id} not found."))
        })?;

        let new_api_key = Self::generate_api_key();
        let new_api_key_hash = Self::hash_secret(&new_api_key);

        let updated_client = self.repo.regenerate_token(id, &new_api_key_hash).await?
            .ok_or_else(|| ApiError::NotFound(format!("OAuth client with ID {id} not found after regeneration.")))?;

        Ok((new_api_key, updated_client.expires_at))
    }

    pub async fn check_rate_limit(&self, oauth_service_id: Uuid, rate_limit_per_minute: i32) -> Result<bool, ApiError> {
        let count = self.repo.count_requests_in_window(oauth_service_id, 60).await?;
        Ok(count < rate_limit_per_minute as i64)
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
    ) -> Result<(), ApiError> {
        self.repo.log_usage(
            oauth_service_id,
            endpoint,
            model,
            tokens_used,
            latency_ms,
            response_status,
            error_message,
        ).await?;
        Ok(())
    }
}
