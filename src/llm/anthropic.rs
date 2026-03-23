//! Anthropic Messages API — [`super::LlmClient`](super::LlmClient) for Claude.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::ToolDefinition;

use super::client::{LlmClient, StreamingLlmClient};
use super::context::Context;
use super::error::LlmError;
use super::types::{AgentResponse, LlmStreamChunk, ToolCall};

// ---------------------------------------------------------------------------
// Wire types (Anthropic Messages API)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct ClaudeRequest {
    pub model: String,
    pub max_tokens: u32,
    pub system: String,
    pub messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ClaudeTool>,
}

#[derive(Serialize, Clone)]
pub struct ClaudeTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Serialize, Clone, Debug)]
pub struct ClaudeMessage {
    pub role: String,
    pub content: ClaudeMessageContent,
}

#[derive(Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ClaudeMessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        is_error: bool,
    },
}

#[derive(Deserialize, Debug)]
pub struct ClaudeResponse {
    pub content: Vec<ClaudeResponseContent>,
    pub stop_reason: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub usage: Option<Usage>,
}

#[derive(Deserialize, Debug)]
pub struct ClaudeResponseContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub input: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Usage {
    #[allow(dead_code)]
    pub input_tokens: u32,
    #[allow(dead_code)]
    pub output_tokens: u32,
}

#[derive(Deserialize, Debug)]
pub struct ClaudeErrorResponse {
    pub error: ClaudeErrorDetail,
}

#[derive(Deserialize, Debug)]
pub struct ClaudeErrorDetail {
    pub message: String,
}

// ---------------------------------------------------------------------------

const ANTHROPIC_URL: &str = "https://api.anthropic.com/v1/messages";

/// Anthropic Claude [`LlmClient`] / [`StreamingLlmClient`].
pub struct AnthropicLlmClient {
    api_key: String,
    http: reqwest::Client,
}

impl AnthropicLlmClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            http: reqwest::Client::new(),
        }
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    async fn post_messages(
        &self,
        ctx: &Context,
        tools: &[ClaudeTool],
    ) -> Result<ClaudeResponse, LlmError> {
        if self.api_key.is_empty() {
            return Err(LlmError::MissingApiKey);
        }

        let request = ClaudeRequest {
            model: ctx.model.clone(),
            max_tokens: ctx.max_tokens,
            system: ctx.system_prompt.clone(),
            messages: ctx.messages.clone(),
            tools: tools.to_vec(),
        };

        let result = self
            .http
            .post(ANTHROPIC_URL)
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        let status = result.status();
        let body = result
            .text()
            .await
            .map_err(|e| LlmError::Http(e.to_string()))?;

        if !status.is_success() {
            return match serde_json::from_str::<ClaudeErrorResponse>(&body) {
                Ok(e) => Err(LlmError::Api {
                    status: status.as_u16(),
                    message: e.error.message,
                }),
                Err(_) => Err(LlmError::Api {
                    status: status.as_u16(),
                    message: body,
                }),
            };
        }

        serde_json::from_str(&body)
            .map_err(|e| LlmError::InvalidResponse(format!("parse error: {e}; body: {body}")))
    }

    fn tools_to_claude(tools: &[ToolDefinition]) -> Vec<ClaudeTool> {
        tools
            .iter()
            .map(|t| ClaudeTool {
                name: t.name.clone(),
                description: t.description.clone(),
                input_schema: t.input_schema.clone(),
            })
            .collect()
    }

    fn response_to_assistant_and_agent(
        resp: &ClaudeResponse,
    ) -> Result<(ClaudeMessage, AgentResponse), LlmError> {
        let mut text_parts: Vec<String> = Vec::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut blocks: Vec<ContentBlock> = Vec::new();

        for content in &resp.content {
            match content.content_type.as_str() {
                "text" => {
                    if let Some(t) = &content.text {
                        if !t.is_empty() {
                            text_parts.push(t.clone());
                            blocks.push(ContentBlock::Text { text: t.clone() });
                        }
                    }
                }
                "tool_use" => {
                    if let (Some(id), Some(name), Some(input)) =
                        (&content.id, &content.name, &content.input)
                    {
                        tool_calls.push(ToolCall {
                            id: id.clone(),
                            name: name.clone(),
                            arguments: input.clone(),
                        });
                        blocks.push(ContentBlock::ToolUse {
                            id: id.clone(),
                            name: name.clone(),
                            input: input.clone(),
                        });
                    }
                }
                _ => {}
            }
        }

        let text = if text_parts.is_empty() {
            None
        } else {
            Some(text_parts.join("\n\n"))
        };

        let assistant_content = if blocks.is_empty() {
            ClaudeMessageContent::Text(String::new())
        } else if blocks.len() == 1 {
            match &blocks[0] {
                ContentBlock::Text { text } => ClaudeMessageContent::Text(text.clone()),
                _ => ClaudeMessageContent::Blocks(blocks),
            }
        } else {
            ClaudeMessageContent::Blocks(blocks)
        };

        let assistant = ClaudeMessage {
            role: "assistant".to_string(),
            content: assistant_content,
        };

        let agent = AgentResponse {
            text,
            tool_calls,
            stop_reason: resp.stop_reason.clone(),
        };

        Ok((assistant, agent))
    }
}

#[async_trait]
impl LlmClient for AnthropicLlmClient {
    async fn chat(
        &self,
        context: &mut Context,
        message: &str,
        tools: &[ToolDefinition],
    ) -> Result<AgentResponse, LlmError> {
        context.push_user_text(message);
        let claude_tools = Self::tools_to_claude(tools);
        let resp = self.post_messages(context, &claude_tools).await?;
        let (assistant, agent) = Self::response_to_assistant_and_agent(&resp)?;
        context.messages.push(assistant);
        Ok(agent)
    }

    async fn continue_with_result(
        &self,
        context: &mut Context,
        tool_call: &ToolCall,
        result: &crate::tools::ToolResult,
        tools: &[ToolDefinition],
    ) -> Result<AgentResponse, LlmError> {
        self.continue_with_results(context, &[(tool_call.clone(), result.clone())], tools)
            .await
    }

    async fn continue_with_results(
        &self,
        context: &mut Context,
        pairs: &[(ToolCall, crate::tools::ToolResult)],
        tools: &[ToolDefinition],
    ) -> Result<AgentResponse, LlmError> {
        let tool_blocks: Vec<ContentBlock> = pairs
            .iter()
            .map(|(_call, tr)| ContentBlock::ToolResult {
                tool_use_id: tr.tool_use_id.clone(),
                content: tr.content.clone(),
                is_error: tr.is_error,
            })
            .collect();

        context.messages.push(ClaudeMessage {
            role: "user".to_string(),
            content: ClaudeMessageContent::Blocks(tool_blocks),
        });

        let claude_tools = Self::tools_to_claude(tools);
        let resp = self.post_messages(context, &claude_tools).await?;
        let (assistant, agent) = Self::response_to_assistant_and_agent(&resp)?;
        context.messages.push(assistant);
        Ok(agent)
    }
}

#[async_trait]
impl StreamingLlmClient for AnthropicLlmClient {
    async fn chat_stream(
        &self,
        _context: &mut Context,
        _message: &str,
        _tools: &[ToolDefinition],
    ) -> Result<futures::stream::BoxStream<'static, Result<LlmStreamChunk, LlmError>>, LlmError>
    {
        Err(LlmError::StreamingNotSupported)
    }

    async fn continue_with_result_stream(
        &self,
        _context: &mut Context,
        _tool_call: &ToolCall,
        _result: &crate::tools::ToolResult,
        _tools: &[ToolDefinition],
    ) -> Result<futures::stream::BoxStream<'static, Result<LlmStreamChunk, LlmError>>, LlmError>
    {
        Err(LlmError::StreamingNotSupported)
    }
}
