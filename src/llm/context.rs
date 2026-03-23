//! Conversation state for LLM providers (native: Anthropic wire format).

#[cfg(not(target_arch = "wasm32"))]
use super::anthropic::ClaudeMessage;
use crate::api::{ChatMessage, MessageRole};

/// Mutable conversation state passed to [`super::LlmClient`](super::LlmClient).
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub struct Context {
    pub system_prompt: String,
    pub model: String,
    pub max_tokens: u32,
    pub(crate) messages: Vec<ClaudeMessage>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Context {
    pub fn new(
        system_prompt: impl Into<String>,
        model: impl Into<String>,
        max_tokens: u32,
    ) -> Self {
        Self {
            system_prompt: system_prompt.into(),
            model: model.into(),
            max_tokens,
            messages: Vec::new(),
        }
    }

    /// Build context from all prior [`ChatMessage`] entries; the latest user turn is supplied to [`super::LlmClient::chat`](super::LlmClient::chat).
    /// Prior turns only: if the last entry is a user message, it is **excluded** so it can be passed
    /// as `message` to [`super::LlmClient::chat`](super::LlmClient::chat) (avoids duplicating the latest user line).
    pub fn from_prior_chat_messages(
        system_prompt: impl Into<String>,
        model: impl Into<String>,
        max_tokens: u32,
        messages: &[ChatMessage],
    ) -> Self {
        let slice = if matches!(messages.last(), Some(m) if m.role == MessageRole::User) {
            messages.len().saturating_sub(1)
        } else {
            messages.len()
        };
        let messages_claude: Vec<ClaudeMessage> = messages[..slice]
            .iter()
            .map(|m| ClaudeMessage {
                role: match m.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                },
                content: super::anthropic::ClaudeMessageContent::Text(m.content.clone()),
            })
            .collect();

        Self {
            system_prompt: system_prompt.into(),
            model: model.into(),
            max_tokens,
            messages: messages_claude,
        }
    }

    pub(crate) fn push_user_text(&mut self, text: &str) {
        self.messages.push(ClaudeMessage {
            role: "user".to_string(),
            content: super::anthropic::ClaudeMessageContent::Text(text.to_string()),
        });
    }
}
