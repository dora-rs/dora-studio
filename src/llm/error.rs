//! Structured errors for LLM providers.

use thiserror::Error;

/// Errors returned by [`crate::llm::LlmClient`](super::LlmClient) implementations.
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("API key is missing; set ANTHROPIC_API_KEY or configure the client")]
    MissingApiKey,

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("failed to parse model response: {0}")]
    Deserialize(#[from] serde_json::Error),

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("empty response from model")]
    EmptyResponse,

    #[error("maximum tool iterations ({0}) exceeded")]
    MaxToolIterations(u32),

    #[error("streaming is not supported for this provider")]
    StreamingNotSupported,

    #[error("conversation state error: {0}")]
    Context(String),
}
