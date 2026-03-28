use crate::types::GatewayConfig;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub fn calculate_config_hash(config: &GatewayConfig) -> u64 {
    let mut hasher = DefaultHasher::new();
    config.hash(&mut hasher);
    hasher.finish()
}

pub fn configs_are_equal(config1: &GatewayConfig, config2: &GatewayConfig) -> bool {
    calculate_config_hash(config1) == calculate_config_hash(config2)
}
