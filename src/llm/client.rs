//! [`LlmClient`] — async trait for chat + tool continuation + optional streaming.

use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::tools::ToolDefinition;

use super::context::Context;
use super::error::LlmError;
use super::types::{AgentResponse, LlmStreamChunk, ToolCall};

/// Async LLM provider: one chat turn and tool-result continuation.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Append `message` as a user turn, run one model request, update `context`, return structured output.
    async fn chat(
        &self,
        context: &mut Context,
        message: &str,
        tools: &[ToolDefinition],
    ) -> Result<AgentResponse, LlmError>;

    /// After executing **one** tool, append the tool result and request the next model turn.
    /// `tools` must be the same schema as used in [`LlmClient::chat`](Self::chat) for this session.
    async fn continue_with_result(
        &self,
        context: &mut Context,
        tool_call: &ToolCall,
        result: &crate::tools::ToolResult,
        tools: &[ToolDefinition],
    ) -> Result<AgentResponse, LlmError>;

    /// After executing **multiple** tools from the same assistant turn, append all tool results in one user message (recommended for Anthropic).
    async fn continue_with_results(
        &self,
        context: &mut Context,
        pairs: &[(ToolCall, crate::tools::ToolResult)],
        tools: &[ToolDefinition],
    ) -> Result<AgentResponse, LlmError>;
}

/// Optional streaming API (providers may return [`LlmError::StreamingNotSupported`] until implemented).
#[async_trait]
pub trait StreamingLlmClient: Send + Sync {
    async fn chat_stream(
        &self,
        context: &mut Context,
        message: &str,
        tools: &[ToolDefinition],
    ) -> Result<BoxStream<'static, Result<LlmStreamChunk, LlmError>>, LlmError>;

    async fn continue_with_result_stream(
        &self,
        context: &mut Context,
        tool_call: &ToolCall,
        result: &crate::tools::ToolResult,
        tools: &[ToolDefinition],
    ) -> Result<BoxStream<'static, Result<LlmStreamChunk, LlmError>>, LlmError>;
}
