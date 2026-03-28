use crate::types::GatewayConfig;
use std::collections::HashSet;

pub fn validate_gateway_config(config: &GatewayConfig) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    let provider_keys: HashSet<&String> = config.providers.iter().map(|p| &p.key).collect();
    for model in &config.models {
        if !provider_keys.contains(&model.provider) {
            errors.push(format!(
                "Model '{}' references non-existent provider '{}'.",
                model.key, model.provider
            ));
        }
    }

    let model_keys: HashSet<&String> = config.models.iter().map(|m| &m.key).collect();
    for pipeline in &config.pipelines {
        for plugin in &pipeline.plugins {
            if let crate::types::PluginConfig::ModelRouter {
                models: router_models,
            } = plugin
            {
                for model_key in router_models {
                    if !model_keys.contains(model_key) {
                        errors.push(format!(
                            "Pipeline '{}'s ModelRouter references non-existent model '{}'.",
                            pipeline.name, model_key
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ModelConfig, Pipeline, PipelineType, PluginConfig, Provider, ProviderType};

    #[test]
    fn test_valid_config() {
        let config = GatewayConfig {
            general: None,
            providers: vec![Provider {
                key: "p1".to_string(),
                r#type: ProviderType::OpenAI,
                api_key: "key1".to_string(),
                params: Default::default(),
            }],
            models: vec![ModelConfig {
                key: "m1".to_string(),
                r#type: "gpt-4".to_string(),
                provider: "p1".to_string(),
                params: Default::default(),
            }],
            pipelines: vec![Pipeline {
                name: "pipe1".to_string(),
                r#type: PipelineType::Chat,
                plugins: vec![PluginConfig::ModelRouter {
                    models: vec!["m1".to_string()],
                }],
            }],
        };
        assert!(validate_gateway_config(&config).is_ok());
    }

    #[test]
    fn test_invalid_model_provider_ref() {
        let config = GatewayConfig {
            general: None,
            providers: vec![Provider {
                key: "p1".to_string(),
                r#type: ProviderType::OpenAI,
                api_key: "key1".to_string(),
                params: Default::default(),
            }],
            models: vec![ModelConfig {
                key: "m1".to_string(),
                r#type: "gpt-4".to_string(),
                provider: "p2_non_existent".to_string(),
                params: Default::default(),
            }],
            pipelines: vec![],
        };
        let result = validate_gateway_config(&config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("references non-existent provider 'p2_non_existent'"));
    }

    #[test]
    fn test_invalid_pipeline_model_ref() {
        let config = GatewayConfig {
            general: None,
            providers: vec![Provider {
                key: "p1".to_string(),
                r#type: ProviderType::OpenAI,
                api_key: "key1".to_string(),
                params: Default::default(),
            }],
            models: vec![ModelConfig {
                key: "m1".to_string(),
                r#type: "gpt-4".to_string(),
                provider: "p1".to_string(),
                params: Default::default(),
            }],
            pipelines: vec![Pipeline {
                name: "pipe1".to_string(),
                r#type: PipelineType::Chat,
                plugins: vec![PluginConfig::ModelRouter {
                    models: vec!["m2_non_existent".to_string()],
                }],
            }],
        };
        let result = validate_gateway_config(&config);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("references non-existent model 'm2_non_existent'"));
    }
}
