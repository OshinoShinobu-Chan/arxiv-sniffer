pub mod deepseek;

#[derive(Debug, Clone)]
/// This struct represents the token usage metrics of an AI response.
pub struct TokenMetrics {
    pub completion_tokens: usize,
    pub reasoning_tokens: usize,
    pub prompt_cache_hit_tokens: usize,
    pub prompt_cache_miss_tokens: usize,
    pub total_tokens: usize,
}

impl std::ops::Add for TokenMetrics {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            completion_tokens: self.completion_tokens + rhs.completion_tokens,
            reasoning_tokens: self.reasoning_tokens + rhs.reasoning_tokens,
            prompt_cache_hit_tokens: self.prompt_cache_hit_tokens + rhs.prompt_cache_hit_tokens,
            prompt_cache_miss_tokens: self.prompt_cache_miss_tokens + rhs.prompt_cache_miss_tokens,
            total_tokens: self.total_tokens + rhs.total_tokens,
        }
    }
}

impl TokenMetrics {
    pub fn new() -> Self {
        Self {
            completion_tokens: 0,
            reasoning_tokens: 0,
            prompt_cache_hit_tokens: 0,
            prompt_cache_miss_tokens: 0,
            total_tokens: 0,
        }
    }
}

pub trait AiClient: Send + Sync {
    #[allow(dead_code)]
    /// Send a chat completion request using plain user prompt text,
    /// and return the assistant text response.
    fn chat_completions(&self, request_string: String) -> Result<String, String>;
    /// Send a chat completion request using plain user prompt text,
    /// and return the parsed JSON response body.
    fn chat_completions_json(&self, request_string: String) -> Result<serde_json::Value, String>;
    /// Get the total token usage metrics of the AI client.
    fn get_token_metrics(&self) -> TokenMetrics;
}
