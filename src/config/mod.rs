use serde::Deserialize;
use std::fs;

pub mod ai;

pub use ai::{AiConfig, AiModelConfig};

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub crawler: CrawlerConfig,
    #[serde(default)]
    pub prompts: PromptsConfig,
    #[serde(default)]
    pub filter: FilterConfig,
    #[serde(default)]
    pub ai: AiConfig,
}

#[derive(Debug, Deserialize)]
pub struct PromptsConfig {
    #[serde(default = "default_prompts_dir")]
    pub dir: String,
}

impl Default for PromptsConfig {
    fn default() -> Self {
        Self {
            dir: default_prompts_dir(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FilterConfig {
    #[serde(default = "default_relevance_threshold")]
    pub relevance_threshold: f64,
    #[serde(default = "default_eval_concurrency")]
    pub eval_concurrency: usize,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            relevance_threshold: default_relevance_threshold(),
            eval_concurrency: default_eval_concurrency(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CrawlerConfig {
    pub interval_secs: u64,
    #[serde(default = "default_crawler_timeout_secs")]
    pub timeout_secs: u64,
    pub user_agent: Option<String>,
}

fn default_crawler_timeout_secs() -> u64 {
    30
}

fn default_prompts_dir() -> String {
    "prompts".to_string()
}

fn default_relevance_threshold() -> f64 {
    85.0
}

fn default_eval_concurrency() -> usize {
    4
}

impl AppConfig {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let raw = fs::read_to_string(path)?;
        let config = toml::from_str::<Self>(&raw)?;
        Ok(config)
    }

    pub fn ai_model_config(&self, model_name: &str) -> Option<&AiModelConfig> {
        self.ai.models.get(model_name)
    }
}
