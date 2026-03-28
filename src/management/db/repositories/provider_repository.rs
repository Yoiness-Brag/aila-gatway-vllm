use serde_json::Value as JsonValue;
use sqlx::{PgPool, Result, query, query_as, types::Uuid};

use crate::management::{
    db::models::Provider,
    dto::{CreateProviderRequest, UpdateProviderRequest},
};

#[derive(Debug)]
pub struct CreateProviderData {
    pub name: String,
    pub provider_type: String,
    pub config_details: JsonValue,
    pub enabled: bool,
}

#[derive(Debug)]
pub struct UpdateProviderData {
    pub name: Option<String>,
    pub config_details: Option<JsonValue>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ProviderRepository {
    pool: PgPool,
}

impl ProviderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        data: &CreateProviderRequest,
        provider_type_str: &str,
        config_json_value: JsonValue,
    ) -> Result<Provider> {
        let new_id = Uuid::new_v4();
        let enabled = data.enabled.unwrap_or(true);
        query_as!(
            Provider,
            r#"
            INSERT INTO hub_llmgateway_providers (id, name, provider_type, config_details, enabled)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, provider_type, config_details, enabled, created_at, updated_at
            "#,
            new_id,
            data.name,
            provider_type_str,
            config_json_value,
            enabled
        )
        .fetch_one(&self.pool)
        .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Provider>> {
        query_as!(
            Provider,
            r#"
            SELECT id, name, provider_type, config_details, enabled, created_at, updated_at
            FROM hub_llmgateway_providers
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Provider>> {
        query_as!(
            Provider,
            r#"
            SELECT id, name, provider_type, config_details, enabled, created_at, updated_at
            FROM hub_llmgateway_providers
            WHERE name = $1
            "#,
            name
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn list(&self) -> Result<Vec<Provider>> {
        query_as!(
            Provider,
            r#"
            SELECT id, name, provider_type, config_details, enabled, created_at, updated_at
            FROM hub_llmgateway_providers
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: Uuid,
        data: &UpdateProviderRequest,
        config_json_value_opt: Option<JsonValue>,
    ) -> Result<Option<Provider>> {
        let current_provider = self
            .find_by_id(id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        let name_to_update = data.name.as_ref().unwrap_or(&current_provider.name);
        let enabled_to_update = data.enabled.unwrap_or(current_provider.enabled);

        let final_config_details: JsonValue = match config_json_value_opt {
            Some(new_val) => new_val,                        // new_val is JsonValue
            None => current_provider.config_details.clone(), // current_provider.config_details is JsonValue, clone it
        };

        query_as!(
            Provider,
            r#"
            UPDATE hub_llmgateway_providers
            SET
                name = $1,
                config_details = $2,
                enabled = $3,
                updated_at = now()
            WHERE id = $4
            RETURNING id, name, provider_type, config_details, enabled, created_at, updated_at
            "#,
            name_to_update,
            final_config_details, // This is Option<JsonValue>
            enabled_to_update,
            id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(&self, id: Uuid) -> Result<u64> {
        let result = query!(
            r#"
            DELETE FROM hub_llmgateway_providers
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }
}
