#[cfg(not(target_arch = "wasm32"))]
use crate::tools::{execute_tool, get_dora_tools};
#[cfg(target_arch = "wasm32")]
use makepad_widgets::Cx;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// Native-only imports
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;
#[cfg(not(target_arch = "wasm32"))]
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

// Global state
#[cfg(not(target_arch = "wasm32"))]
static TOKIO_RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);
#[cfg(not(target_arch = "wasm32"))]
static REQUEST_SENDER: Mutex<Option<UnboundedSender<Vec<ChatMessage>>>> = Mutex::new(None);
static API_KEY: Mutex<String> = Mutex::new(String::new());

// Pending response for polling
static PENDING_RESPONSE: Mutex<Option<ChatResponse>> = Mutex::new(None);

/// Check if there's a pending response from the API
pub fn take_pending_response() -> Option<ChatResponse> {
    PENDING_RESPONSE.lock().unwrap().take()
}

/// System prompt for the Dora assistant (native with tools)
#[cfg(not(target_arch = "wasm32"))]
const SYSTEM_PROMPT: &str = r#"You are Dora Studio Assistant. Be extremely concise and succinct.

Rules:
- Give short, direct answers
- No unnecessary explanations or preambles
- Only provide details when specifically asked
- Use bullet points for lists
- Skip pleasantries

You have tools for: dora dataflows (list/start/stop/destroy), file operations (read/write), shell commands, directory browsing.

Use tools proactively. Show results briefly."#;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone)]
pub enum ChatResponse {
    Message(String),
    ToolExecution(String), // Intermediate message showing tool execution
    Error(String),
}

/// Set the API key for Claude
pub fn set_api_key(key: String) {
    *API_KEY.lock().unwrap() = key;
}

/// Get the current API key
pub fn get_api_key() -> String {
    API_KEY.lock().unwrap().clone()
}

/// Initialize API key from environment variable
pub fn init_api_key_from_env() {
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        if !key.is_empty() {
            eprintln!("[API] Loaded API key from env ({} chars)", key.len());
            set_api_key(key);
        } else {
            eprintln!("[API] ANTHROPIC_API_KEY env var is empty");
        }
    } else {
        eprintln!("[API] ANTHROPIC_API_KEY env var not set");
    }
}

/// Initialize the async runtime for API calls (native only)
#[cfg(not(target_arch = "wasm32"))]
pub fn start_api_runtime() {
    // Use a single lock to check and initialize atomically
    let mut rt_lock = TOKIO_RUNTIME.lock().unwrap();
    if rt_lock.is_some() {
        return;
    }

    // Mark as initialized immediately to prevent race conditions
    *rt_lock = Some(Runtime::new().expect("marker runtime"));
    drop(rt_lock); // Release the lock before spawning

    // Try to load API key from environment (fallback if startup didn't run)
    init_api_key_from_env();

    // Set up channel
    let (sender, mut receiver) = unbounded_channel::<Vec<ChatMessage>>();
    *REQUEST_SENDER.lock().unwrap() = Some(sender);

    // Create runtime and run it on a background thread
    std::thread::spawn(move || {
        let rt = Runtime::new().expect("Failed to create Tokio runtime");

        rt.block_on(async {
            eprintln!("[API] Runtime started, waiting for requests...");
            while let Some(messages) = receiver.recv().await {
                eprintln!("[API] Received request with {} messages", messages.len());
                eprintln!("[API] API key length: {}", get_api_key().len());
                let response = call_claude_api_with_tools(messages).await;
                eprintln!(
                    "[API] Got response: {:?}",
                    match &response {
                        ChatResponse::Message(s) => format!("Message({} chars)", s.len()),
                        ChatResponse::ToolExecution(s) => format!("Tool: {}", s),
                        ChatResponse::Error(e) => format!("Error: {}", e),
                    }
                );
                // Store response for polling instead of post_action
                *PENDING_RESPONSE.lock().unwrap() = Some(response);
                eprintln!("[API] Response stored for polling");
            }
        });
    });
}

/// Submit a chat request to the Claude API (native)
#[cfg(not(target_arch = "wasm32"))]
pub fn submit_chat_request(messages: Vec<ChatMessage>) {
    eprintln!("[API] submit_chat_request called");
    // Ensure runtime is started
    start_api_runtime();
    eprintln!("[API] runtime started");

    eprintln!("[API] acquiring REQUEST_SENDER lock");
    if let Some(sender) = REQUEST_SENDER.lock().unwrap().as_ref() {
        eprintln!("[API] sending message");
        let _ = sender.send(messages);
        eprintln!("[API] message sent");
    } else {
        eprintln!("[API] no sender available!");
    }
    eprintln!("[API] submit_chat_request complete");
}

/// Submit a chat request to the Claude API (WASM)
#[cfg(target_arch = "wasm32")]
pub fn submit_chat_request(messages: Vec<ChatMessage>) {
    wasm_bindgen_futures::spawn_local(async move {
        let response = call_claude_api_simple(messages).await;
        Cx::post_action(response);
    });
}

// ============================================================================
// Claude API Request/Response Structures
// ============================================================================

/// Claude API request with tools (native only)
#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<ClaudeTool>,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Clone)]
struct ClaudeTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Clone)]
struct ClaudeMessage {
    role: String,
    content: ClaudeMessageContent,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Clone)]
#[serde(untagged)]
enum ClaudeMessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Serialize, Clone)]
#[serde(tag = "type")]
enum ContentBlock {
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

/// Claude API response structure
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ClaudeResponse {
    content: Vec<ClaudeResponseContent>,
    stop_reason: Option<String>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct ClaudeResponseContent {
    #[serde(rename = "type")]
    content_type: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    input: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Deserialize)]
struct ClaudeErrorResponse {
    error: ClaudeErrorDetail,
}

#[derive(Deserialize)]
struct ClaudeErrorDetail {
    message: String,
}

// ============================================================================
// WASM Simple API Call (no tools)
// ============================================================================

#[cfg(target_arch = "wasm32")]
const WASM_SYSTEM_PROMPT: &str = "You are Dora Studio Assistant. Be extremely concise. Short answers only. No unnecessary explanations. Note: Tools unavailable in web version - use desktop app for full features.";

#[cfg(target_arch = "wasm32")]
async fn call_claude_api_simple(messages: Vec<ChatMessage>) -> ChatResponse {
    let api_key = get_api_key();

    if api_key.is_empty() {
        return ChatResponse::Error("Please enter your Claude API key in the header".to_string());
    }

    let client = reqwest::Client::new();

    // Convert messages to Claude format (simple text only)
    let claude_messages: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| {
            serde_json::json!({
                "role": match m.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                },
                "content": m.content
            })
        })
        .collect();

    let request = serde_json::json!({
        "model": "claude-sonnet-4-20250514",
        "max_tokens": 4096,
        "system": WASM_SYSTEM_PROMPT,
        "messages": claude_messages
    });

    let result = client
        .post("https://api.anthropic.com/v1/messages")
        .header("Content-Type", "application/json")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&request)
        .send()
        .await;

    match result {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if status.is_success() {
                match serde_json::from_str::<ClaudeResponse>(&body) {
                    Ok(claude_response) => {
                        let text: String = claude_response
                            .content
                            .iter()
                            .filter_map(|c| c.text.clone())
                            .collect::<Vec<_>>()
                            .join("\n");

                        if text.is_empty() {
                            ChatResponse::Error("Empty response from Claude".to_string())
                        } else {
                            ChatResponse::Message(text)
                        }
                    }
                    Err(e) => ChatResponse::Error(format!("Failed to parse response: {}", e)),
                }
            } else {
                match serde_json::from_str::<ClaudeErrorResponse>(&body) {
                    Ok(error_response) => {
                        ChatResponse::Error(format!("API Error: {}", error_response.error.message))
                    }
                    Err(_) => ChatResponse::Error(format!("API Error ({}): {}", status, body)),
                }
            }
        }
        Err(e) => ChatResponse::Error(format!("Network error: {}", e)),
    }
}

// ============================================================================
// Native API Call Implementation with Tool Loop
// ============================================================================

/// Call Claude API with tools support - implements the agentic loop
#[cfg(not(target_arch = "wasm32"))]
async fn call_claude_api_with_tools(messages: Vec<ChatMessage>) -> ChatResponse {
    let api_key = get_api_key();

    if api_key.is_empty() {
        return ChatResponse::Error("Please enter your Claude API key in the header".to_string());
    }

    let client = reqwest::Client::new();

    // Convert tools to Claude format
    let tools: Vec<ClaudeTool> = get_dora_tools()
        .into_iter()
        .map(|t| ClaudeTool {
            name: t.name,
            description: t.description,
            input_schema: t.input_schema,
        })
        .collect();

    // Convert initial messages to Claude format
    let mut claude_messages: Vec<ClaudeMessage> = messages
        .iter()
        .map(|m| ClaudeMessage {
            role: match m.role {
                MessageRole::User => "user".to_string(),
                MessageRole::Assistant => "assistant".to_string(),
            },
            content: ClaudeMessageContent::Text(m.content.clone()),
        })
        .collect();

    // Collect all text responses and tool executions
    let mut final_response = String::new();
    let mut iteration = 0;
    const MAX_ITERATIONS: u32 = 10; // Prevent infinite loops

    loop {
        iteration += 1;
        eprintln!("[API] Iteration {}", iteration);
        if iteration > MAX_ITERATIONS {
            final_response.push_str("\n\n[Reached maximum tool iterations]");
            break;
        }

        let request = ClaudeRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 4096,
            system: SYSTEM_PROMPT.to_string(),
            messages: claude_messages.clone(),
            tools: tools.clone(),
        };

        eprintln!("[API] Sending HTTP request...");
        let result = client
            .post("https://api.anthropic.com/v1/messages")
            .header("Content-Type", "application/json")
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await;

        let response = match result {
            Ok(resp) => resp,
            Err(e) => return ChatResponse::Error(format!("Network error: {}", e)),
        };

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return match serde_json::from_str::<ClaudeErrorResponse>(&body) {
                Ok(error_response) => {
                    ChatResponse::Error(format!("API Error: {}", error_response.error.message))
                }
                Err(_) => ChatResponse::Error(format!("API Error ({}): {}", status, body)),
            };
        }

        let claude_response: ClaudeResponse = match serde_json::from_str(&body) {
            Ok(r) => r,
            Err(e) => {
                return ChatResponse::Error(format!(
                    "Failed to parse response: {}\nBody: {}",
                    e, body
                ))
            }
        };

        // Process response content
        let mut tool_uses: Vec<(String, String, serde_json::Value)> = Vec::new();
        let mut has_text = false;

        for content in &claude_response.content {
            match content.content_type.as_str() {
                "text" => {
                    if let Some(text) = &content.text {
                        if !text.is_empty() {
                            if !final_response.is_empty() {
                                final_response.push_str("\n\n");
                            }
                            final_response.push_str(text);
                            has_text = true;
                        }
                    }
                }
                "tool_use" => {
                    if let (Some(id), Some(name), Some(input)) =
                        (&content.id, &content.name, &content.input)
                    {
                        tool_uses.push((id.clone(), name.clone(), input.clone()));
                    }
                }
                _ => {}
            }
        }

        // Check if we should stop
        let stop_reason = claude_response.stop_reason.as_deref().unwrap_or("");
        if stop_reason == "end_turn" || (tool_uses.is_empty() && has_text) {
            break;
        }

        // Execute tools if any
        if !tool_uses.is_empty() {
            // Build assistant message with tool uses
            let tool_use_blocks: Vec<ContentBlock> = tool_uses
                .iter()
                .map(|(id, name, input)| ContentBlock::ToolUse {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                })
                .collect();

            // Add any text that came with tool uses
            let mut assistant_blocks: Vec<ContentBlock> = Vec::new();
            for content in &claude_response.content {
                if content.content_type == "text" {
                    if let Some(text) = &content.text {
                        if !text.is_empty() {
                            assistant_blocks.push(ContentBlock::Text { text: text.clone() });
                        }
                    }
                }
            }
            assistant_blocks.extend(tool_use_blocks);

            claude_messages.push(ClaudeMessage {
                role: "assistant".to_string(),
                content: ClaudeMessageContent::Blocks(assistant_blocks),
            });

            // Execute tools and collect results
            let mut tool_results: Vec<ContentBlock> = Vec::new();

            for (id, name, input) in &tool_uses {
                // Add tool execution info to response
                if !final_response.is_empty() {
                    final_response.push_str("\n\n");
                }
                final_response.push_str(&format!("🔧 Executing: {}", name));

                let result = execute_tool(name, id, input);

                // Show result preview in final response
                let preview = if result.content.len() > 200 {
                    format!("{}...", &result.content[..200])
                } else {
                    result.content.clone()
                };

                if result.is_error {
                    final_response.push_str(&format!("\n❌ Error: {}", preview));
                } else {
                    final_response.push_str(&format!("\n✅ Result: {}", preview));
                }

                tool_results.push(ContentBlock::ToolResult {
                    tool_use_id: result.tool_use_id,
                    content: result.content,
                    is_error: result.is_error,
                });
            }

            // Add tool results as user message
            claude_messages.push(ClaudeMessage {
                role: "user".to_string(),
                content: ClaudeMessageContent::Blocks(tool_results),
            });
        } else {
            // No tools and no clear end - break to avoid infinite loop
            break;
        }
    }

    if final_response.is_empty() {
        ChatResponse::Error("Empty response from Claude".to_string())
    } else {
        ChatResponse::Message(final_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // ChatMessage Tests
    // ============================================================================

    #[test]
    fn test_chat_message_serialization() {
        let msg = ChatMessage {
            role: MessageRole::User,
            content: "Hello, Claude!".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
        assert!(json.contains("\"content\":\"Hello, Claude!\""));
    }

    #[test]
    fn test_chat_message_deserialization() {
        let json = r#"{"role":"assistant","content":"Hello!"}"#;
        let msg: ChatMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.role, MessageRole::Assistant);
        assert_eq!(msg.content, "Hello!");
    }

    #[test]
    fn test_message_role_serialization() {
        let user_json = serde_json::to_string(&MessageRole::User).unwrap();
        let assistant_json = serde_json::to_string(&MessageRole::Assistant).unwrap();
        assert_eq!(user_json, "\"user\"");
        assert_eq!(assistant_json, "\"assistant\"");
    }

    #[test]
    fn test_message_role_deserialization() {
        let user: MessageRole = serde_json::from_str("\"user\"").unwrap();
        let assistant: MessageRole = serde_json::from_str("\"assistant\"").unwrap();
        assert_eq!(user, MessageRole::User);
        assert_eq!(assistant, MessageRole::Assistant);
    }

    // ============================================================================
    // API Key Management Tests
    // ============================================================================

    #[test]
    fn test_set_and_get_api_key() {
        let test_key = "sk-ant-test-key-12345";
        set_api_key(test_key.to_string());
        assert_eq!(get_api_key(), test_key);
        // Reset for other tests
        set_api_key(String::new());
    }

    #[test]
    fn test_empty_api_key() {
        set_api_key(String::new());
        assert!(get_api_key().is_empty());
    }

    #[test]
    fn test_api_key_overwrites_previous() {
        set_api_key("first-key".to_string());
        set_api_key("second-key".to_string());
        assert_eq!(get_api_key(), "second-key");
        // Reset for other tests
        set_api_key(String::new());
    }

    // ============================================================================
    // ChatResponse Tests
    // ============================================================================

    #[test]
    fn test_chat_response_message() {
        let response = ChatResponse::Message("Hello from Claude".to_string());
        match response {
            ChatResponse::Message(text) => assert_eq!(text, "Hello from Claude"),
            ChatResponse::ToolExecution(_) => panic!("Expected Message variant"),
            ChatResponse::Error(_) => panic!("Expected Message variant"),
        }
    }

    #[test]
    fn test_chat_response_error() {
        let response = ChatResponse::Error("API Error: Invalid key".to_string());
        match response {
            ChatResponse::Error(err) => assert!(err.contains("Invalid key")),
            ChatResponse::Message(_) => panic!("Expected Error variant"),
            ChatResponse::ToolExecution(_) => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_chat_response_tool_execution() {
        let response = ChatResponse::ToolExecution("Running dora_list".to_string());
        match response {
            ChatResponse::ToolExecution(msg) => assert!(msg.contains("dora_list")),
            ChatResponse::Message(_) => panic!("Expected ToolExecution variant"),
            ChatResponse::Error(_) => panic!("Expected ToolExecution variant"),
        }
    }

    // ============================================================================
    // Claude Response Structure Tests
    // ============================================================================

    #[test]
    fn test_claude_response_deserialization() {
        let json = r#"{"content":[{"type":"text","text":"Hello!"}]}"#;
        let response: ClaudeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.content.len(), 1);
        assert_eq!(response.content[0].text, Some("Hello!".to_string()));
    }

    #[test]
    fn test_claude_response_empty_content() {
        let json = r#"{"content":[]}"#;
        let response: ClaudeResponse = serde_json::from_str(json).unwrap();
        assert!(response.content.is_empty());
    }

    #[test]
    fn test_claude_error_response_deserialization() {
        let json = r#"{"error":{"message":"Invalid API key"}}"#;
        let response: ClaudeErrorResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.error.message, "Invalid API key");
    }

    // ============================================================================
    // Native-only Tests (require tool support structures)
    // ============================================================================

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_claude_request_serialization() {
        let request = ClaudeRequest {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 4096,
            system: "You are helpful".to_string(),
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: ClaudeMessageContent::Text("Hello".to_string()),
            }],
            tools: vec![],
        };
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"model\":\"claude-sonnet-4-20250514\""));
        assert!(json.contains("\"max_tokens\":4096"));
        assert!(json.contains("\"role\":\"user\""));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_message_to_claude_format() {
        let messages = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: "Hi there!".to_string(),
            },
        ];

        let claude_messages: Vec<ClaudeMessage> = messages
            .iter()
            .map(|m| ClaudeMessage {
                role: match m.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                },
                content: ClaudeMessageContent::Text(m.content.clone()),
            })
            .collect();

        assert_eq!(claude_messages.len(), 2);
        assert_eq!(claude_messages[0].role, "user");
        assert_eq!(claude_messages[1].role, "assistant");
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_call_claude_api_without_key() {
        // Ensure API key is empty
        set_api_key(String::new());

        let messages = vec![ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
        }];

        let response = call_claude_api_with_tools(messages).await;
        match response {
            ChatResponse::Error(err) => {
                assert!(err.contains("API key"));
            }
            ChatResponse::Message(_) => panic!("Expected error for missing API key"),
            ChatResponse::ToolExecution(_) => panic!("Expected error for missing API key"),
        }
    }

    #[test]
    fn test_multiple_messages_conversion() {
        let conversation = vec![
            ChatMessage {
                role: MessageRole::User,
                content: "What is Rust?".to_string(),
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: "Rust is a systems programming language.".to_string(),
            },
            ChatMessage {
                role: MessageRole::User,
                content: "Tell me more.".to_string(),
            },
        ];

        assert_eq!(conversation.len(), 3);
        assert_eq!(conversation[0].role, MessageRole::User);
        assert_eq!(conversation[1].role, MessageRole::Assistant);
        assert_eq!(conversation[2].role, MessageRole::User);
    }
}
