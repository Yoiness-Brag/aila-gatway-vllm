use crate::types::{GatewayConfig, ModelConfig, Pipeline, PipelineType, PluginConfig, Provider};
use serde::Deserialize;
use std::sync::OnceLock;

pub static TRACE_CONTENT_ENABLED: OnceLock<bool> = OnceLock::new();

#[derive(Deserialize, Debug)]
struct YamlCompatiblePipeline {
    name: String,
    r#type: PipelineType,
    #[serde(with = "serde_yaml::with::singleton_map_recursive")]
    plugins: Vec<PluginConfig>,
    #[serde(default = "default_enabled_true_lib")]
    #[allow(dead_code)]
    enabled: bool,
}

fn default_enabled_true_lib() -> bool {
    true
}

#[derive(Deserialize, Debug)]
struct YamlRoot {
    #[serde(default)]
    providers: Vec<Provider>,
    #[serde(default)]
    models: Vec<ModelConfig>,
    #[serde(default)]
    pipelines: Vec<YamlCompatiblePipeline>,
}

fn substitute_env_vars(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    use std::env;

    let mut result = content.to_string();

    let mut start_pos = 0;
    while let Some(start) = result[start_pos..].find("${") {
        let actual_start = start_pos + start;
        if let Some(end) = result[actual_start + 2..].find('}') {
            let actual_end = actual_start + 2 + end;
            let var_name = &result[actual_start + 2..actual_end];

            match env::var(var_name) {
                Ok(value) => {
                    result.replace_range(actual_start..actual_end + 1, &value);
                    start_pos = actual_start + value.len();
                }
                Err(_) => {
                    return Err(format!("Environment variable '{var_name}' not found").into());
                }
            }
        } else {
            start_pos = actual_start + 2;
        }
    }

    Ok(result)
}

pub fn load_config(path: &str) -> Result<GatewayConfig, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string(path)?;
    let contents_with_env = substitute_env_vars(&contents)?;
    let yaml_root: YamlRoot = serde_yaml::from_str(&contents_with_env)?;

    let gateway_config = GatewayConfig {
        providers: yaml_root.providers,
        models: yaml_root.models,
        pipelines: yaml_root
            .pipelines
            .into_iter()
            .map(|p_yaml| {
                Pipeline {
                    name: p_yaml.name,
                    r#type: p_yaml.r#type,
                    plugins: p_yaml.plugins,
                }
            })
            .collect(),
        general: None,
    };
    let _ = TRACE_CONTENT_ENABLED.set(
        gateway_config
            .general
            .as_ref()
            .is_none_or(|g| g.trace_content_enabled),
    );

    Ok(gateway_config)
}

fn parse_env_var_bool(var: &str) -> Option<bool> {
    match var.to_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub fn get_trace_content_enabled() -> bool {
    if let Ok(env_value) = std::env::var("TRACE_CONTENT_ENABLED") {
        if let Some(val) = parse_env_var_bool(&env_value) {
            return val;
        }
    }
    *TRACE_CONTENT_ENABLED.get_or_init(|| true)
}
