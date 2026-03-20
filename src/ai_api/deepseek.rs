use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::{AiClient, TokenMetrics};
use crate::config::ai::deepseek::DeepSeekRequestConfig;
use crate::config::AiModelConfig;

#[derive(Debug)]
pub enum DeepSeekError {
    Http(reqwest::Error),
    Api {
        status: u16,
    },
    InvalidJsonResponse {
        status: u16,
        source: serde_json::Error,
    },
}

impl Display for DeepSeekError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(err) => write!(f, "http error: {:?}", err),
            Self::Api { status } => write!(f, "api error (status={})", status),
            Self::InvalidJsonResponse { status, source } => {
                write!(f, "invalid json response (status={}): {}", status, source)
            }
        }
    }
}

impl std::error::Error for DeepSeekError {}

#[derive(Debug, Clone)]
pub struct DeepSeekClient {
    http: Client,
    api_key: String,
    model_config: DeepSeekModelConfig,
    token_metrics: Arc<Mutex<TokenMetrics>>,
}

#[derive(Debug, Clone)]
pub struct DeepSeekModelConfig {
    pub endpoint: String,
    pub system_prompt: String,
    pub timeout_secs: u64,
    pub request: DeepSeekRequestConfig,
}

impl DeepSeekClient {
    pub fn new(api_key: impl Into<String>, model_config: DeepSeekModelConfig) -> Self {
        Self {
            http: Self::build_http_client(model_config.timeout_secs),
            api_key: api_key.into(),
            model_config,
            token_metrics: Arc::new(Mutex::new(TokenMetrics::new())),
        }
    }
    /// Send a chat completion request to DeepSeek and return the raw JSON response.
    pub fn chat_completions(
        &self,
        request: &ChatCompletionsRequest,
    ) -> Result<Value, DeepSeekError> {
        let response = self
            .http
            .post(&self.model_config.endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .bearer_auth(&self.api_key)
            .json(request)
            .send()
            .map_err(DeepSeekError::Http)?;

        let status = response.status();
        let body_bytes = response.bytes().map_err(DeepSeekError::Http)?;

        if !status.is_success() {
            return Err(DeepSeekError::Api {
                status: status.as_u16(),
            });
        }
        serde_json::from_slice(&body_bytes).map_err(|source| DeepSeekError::InvalidJsonResponse {
            status: status.as_u16(),
            source,
        })
    }

    fn update_token_metrics_from_response(&self, response_json: &Value) {
        let usage = response_json
            .get("usage")
            .and_then(|v| serde_json::from_value::<DeepSeekUsage>(v.clone()).ok());

        let Some(usage) = usage else {
            return;
        };

        let delta = TokenMetrics {
            completion_tokens: usage.completion_tokens,
            reasoning_tokens: usage
                .completion_tokens_details
                .as_ref()
                .map(|d| d.reasoning_tokens)
                .unwrap_or(0),
            prompt_cache_hit_tokens: usage.prompt_cache_hit_tokens,
            prompt_cache_miss_tokens: usage.prompt_cache_miss_tokens,
            total_tokens: usage.total_tokens,
        };

        if let Ok(mut metrics) = self.token_metrics.lock() {
            *metrics = metrics.clone() + delta;
        }
    }

    fn extract_assistant_text(response_json: &Value) -> Result<String, String> {
        response_json
            .get("choices")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                "invalid deepseek response: missing choices[0].message.content".to_string()
            })
    }

    pub fn from_ai_model_config(
        active_name: &str,
        model_cfg: &AiModelConfig,
    ) -> DeepSeekModelConfig {
        let mut request = model_cfg.request.clone();
        if request.model.trim().is_empty() {
            request.model = active_name.to_string();
        }

        DeepSeekModelConfig {
            endpoint: model_cfg.endpoint.clone(),
            system_prompt: model_cfg.system_prompt.clone(),
            timeout_secs: model_cfg.timeout_secs,
            request,
        }
    }

    fn build_http_client(timeout_secs: u64) -> Client {
        Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .expect("failed to build reqwest blocking client for deepseek")
    }

    fn build_request(&self, user_question: &str) -> ChatCompletionsRequest {
        let cfg = &self.model_config;
        ChatCompletionsRequest {
            messages: vec![
                ChatMessage {
                    content: cfg.system_prompt.clone(),
                    role: "system".to_string(),
                },
                ChatMessage {
                    content: user_question.to_string(),
                    role: "user".to_string(),
                },
            ],
            model: cfg.request.model.clone(),
            thinking: ThinkingConfig {
                thinking_type: cfg.request.thinking_type.clone(),
            },
            frequency_penalty: cfg.request.frequency_penalty,
            max_tokens: cfg.request.max_tokens,
            presence_penalty: cfg.request.presence_penalty,
            response_format: ResponseFormat {
                response_type: cfg.request.response_format_type.clone(),
            },
            stop: cfg.request.stop.as_ref().map(|v| serde_json::json!(v)),
            stream: cfg.request.stream,
            stream_options: cfg.request.stream_options.clone(),
            temperature: cfg.request.temperature,
            top_p: cfg.request.top_p,
            tools: cfg.request.tools.clone(),
            tool_choice: serde_json::json!(cfg.request.tool_choice),
            logprobs: cfg.request.logprobs,
            top_logprobs: cfg.request.top_logprobs,
        }
    }
}

impl AiClient for DeepSeekClient {
    fn chat_completions(&self, request_string: String) -> Result<String, String> {
        let request = self.build_request(request_string.trim());
        let response_json = DeepSeekClient::chat_completions(self, &request)
            .map_err(|err| format!("deepseek request failed: {}", err))?;
        self.update_token_metrics_from_response(&response_json);
        Self::extract_assistant_text(&response_json)
    }

    fn chat_completions_json(&self, request_string: String) -> Result<Value, String> {
        let mut request = self.build_request(request_string.trim());
        request.response_format = ResponseFormat {
            response_type: "json_object".to_string(),
        };

        let response_json = DeepSeekClient::chat_completions(self, &request)
            .map_err(|err| format!("deepseek request failed: {}", err))?;
        self.update_token_metrics_from_response(&response_json);

        let content = Self::extract_assistant_text(&response_json)?;
        serde_json::from_str::<Value>(&content)
            .map_err(|err| format!("assistant content is not valid json: {}", err))
    }

    fn get_token_metrics(&self) -> TokenMetrics {
        self.token_metrics
            .lock()
            .map(|m| m.clone())
            .unwrap_or_else(|_| TokenMetrics::new())
    }
}

#[derive(Debug, Deserialize)]
struct DeepSeekUsage {
    completion_tokens: usize,
    #[serde(default)]
    completion_tokens_details: Option<DeepSeekCompletionTokensDetails>,
    prompt_cache_hit_tokens: usize,
    prompt_cache_miss_tokens: usize,
    total_tokens: usize,
}

#[derive(Debug, Deserialize)]
struct DeepSeekCompletionTokensDetails {
    #[serde(default)]
    reasoning_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub content: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub thinking_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub response_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionsRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub thinking: ThinkingConfig,
    pub frequency_penalty: f32,
    pub max_tokens: u32,
    pub presence_penalty: f32,
    pub response_format: ResponseFormat,
    pub stop: Option<Value>,
    pub stream: bool,
    pub stream_options: Option<Value>,
    pub temperature: f32,
    pub top_p: f32,
    pub tools: Option<Value>,
    pub tool_choice: Value,
    pub logprobs: bool,
    pub top_logprobs: Option<u32>,
}
