//! Shared types for LLM requests and responses.

use serde::{Deserialize, Serialize};

/// A tool invocation requested by the model (Anthropic `tool_use`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// One round-trip response from the model (text, tool calls, or both).
#[derive(Debug, Clone, Default)]
pub struct AgentResponse {
    /// Assistant text segments in this turn (may be empty if only tools).
    pub text: Option<String>,
    /// Tool uses requested in this turn.
    pub tool_calls: Vec<ToolCall>,
    /// Provider stop reason when available (e.g. `end_turn`, `tool_use`).
    pub stop_reason: Option<String>,
}

impl AgentResponse {
    pub fn text_only(text: String) -> Self {
        Self {
            text: Some(text),
            tool_calls: vec![],
            stop_reason: None,
        }
    }

    pub fn tool_calls_only(calls: Vec<ToolCall>) -> Self {
        Self {
            text: None,
            tool_calls: calls,
            stop_reason: None,
        }
    }

    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }
}

/// Streaming events for [`crate::llm::StreamingLlmClient`](super::StreamingLlmClient).
#[derive(Debug, Clone)]
pub enum LlmStreamChunk {
    /// Incremental text delta.
    TextDelta(String),
    /// Model finished this stream (caller should not expect more chunks).
    Done,
}
