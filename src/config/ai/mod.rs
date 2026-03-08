use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

pub mod deepseek;

use deepseek::DeepSeekRequestConfig;

#[derive(Debug, Deserialize, Clone)]
pub struct AiConfig {
    #[serde(default = "default_ai_models")]
    pub models: HashMap<String, AiModelConfig>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            models: default_ai_models(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct AiModelConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_deepseek_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    /// DeepSeek-specific request defaults.
    ///
    /// For future providers, add provider-specific config submodules
    /// and extra typed fields on this struct.
    #[serde(default)]
    pub request: DeepSeekRequestConfig,
    /// Optional provider extension data reserved for future models.
    #[serde(default)]
    pub provider_options: Option<Value>,
}

fn default_provider() -> String {
    "deepseek".to_string()
}

fn default_deepseek_endpoint() -> String {
    "https://api.deepseek.com/chat/completions".to_string()
}

fn default_system_prompt() -> String {
    "You are a helpful assistant".to_string()
}

fn default_timeout_secs() -> u64 {
    60
}

fn default_ai_models() -> HashMap<String, AiModelConfig> {
    let mut models = HashMap::new();
    models.insert(
        "deepseek-chat".to_string(),
        AiModelConfig {
            provider: default_provider(),
            endpoint: default_deepseek_endpoint(),
            system_prompt: default_system_prompt(),
            timeout_secs: default_timeout_secs(),
            request: DeepSeekRequestConfig::default(),
            provider_options: None,
        },
    );
    models.insert(
        "deepseek-reasoner".to_string(),
        AiModelConfig {
            provider: default_provider(),
            endpoint: default_deepseek_endpoint(),
            system_prompt: default_system_prompt(),
            timeout_secs: default_timeout_secs(),
            request: DeepSeekRequestConfig {
                model: "deepseek-reasoner".to_string(),
                thinking_type: "enabled".to_string(),
                ..DeepSeekRequestConfig::default()
            },
            provider_options: None,
        },
    );
    models
}
