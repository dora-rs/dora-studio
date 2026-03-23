//! LLM provider abstraction: async [`LlmClient`], Anthropic implementation, errors, and types.

mod client;
mod context;
mod error;
mod types;

#[cfg(not(target_arch = "wasm32"))]
pub mod anthropic;

pub use client::{LlmClient, StreamingLlmClient};
pub use context::Context;
pub use error::LlmError;
pub use types::{AgentResponse, LlmStreamChunk, ToolCall};

#[cfg(not(target_arch = "wasm32"))]
pub use anthropic::AnthropicLlmClient;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::LlmError;

    #[test]
    fn llm_error_display_missing_key() {
        let s = LlmError::MissingApiKey.to_string();
        assert!(s.contains("API key"), "{}", s);
    }
}
