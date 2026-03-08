use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct DeepSeekRequestConfig {
    #[serde(default = "default_model_name")]
    pub model: String,
    #[serde(default = "default_thinking_type")]
    pub thinking_type: String,
    #[serde(default = "default_frequency_penalty")]
    pub frequency_penalty: f32,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_presence_penalty")]
    pub presence_penalty: f32,
    #[serde(default = "default_response_format_type")]
    pub response_format_type: String,
    #[serde(default)]
    pub stop: Option<Vec<String>>,
    #[serde(default = "default_stream")]
    pub stream: bool,
    #[serde(default)]
    pub stream_options: Option<Value>,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default)]
    pub tools: Option<Value>,
    #[serde(default = "default_tool_choice")]
    pub tool_choice: String,
    #[serde(default = "default_logprobs")]
    pub logprobs: bool,
    #[serde(default)]
    pub top_logprobs: Option<u32>,
}

impl Default for DeepSeekRequestConfig {
    fn default() -> Self {
        Self {
            model: default_model_name(),
            thinking_type: default_thinking_type(),
            frequency_penalty: default_frequency_penalty(),
            max_tokens: default_max_tokens(),
            presence_penalty: default_presence_penalty(),
            response_format_type: default_response_format_type(),
            stop: None,
            stream: default_stream(),
            stream_options: None,
            temperature: default_temperature(),
            top_p: default_top_p(),
            tools: None,
            tool_choice: default_tool_choice(),
            logprobs: default_logprobs(),
            top_logprobs: None,
        }
    }
}

fn default_model_name() -> String {
    "deepseek-chat".to_string()
}

fn default_thinking_type() -> String {
    "disabled".to_string()
}

fn default_frequency_penalty() -> f32 {
    0.0
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_presence_penalty() -> f32 {
    0.0
}

fn default_response_format_type() -> String {
    "text".to_string()
}

fn default_stream() -> bool {
    false
}

fn default_temperature() -> f32 {
    1.0
}

fn default_top_p() -> f32 {
    1.0
}

fn default_tool_choice() -> String {
    "none".to_string()
}

fn default_logprobs() -> bool {
    false
}
