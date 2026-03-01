//! Mock implementations for testing
//!
//! Provides mock versions of external dependencies for isolated testing.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

// ============================================================================
// MockDoraClient
// ============================================================================

/// Mock implementation of DoraClient for testing
pub struct MockDoraClient {
    dataflows: Arc<Mutex<Vec<DataflowEntry>>>,
    call_log: Arc<Mutex<Vec<String>>>,
    start_result: Arc<Mutex<Option<Result<Uuid, String>>>>,
}

impl MockDoraClient {
    pub fn new() -> Self {
        Self {
            dataflows: Arc::new(Mutex::new(Vec::new())),
            call_log: Arc::new(Mutex::new(Vec::new())),
            start_result: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_dataflows(&self, flows: Vec<DataflowEntry>) {
        *self.dataflows.lock().unwrap() = flows;
    }

    pub fn expect_start_returns(&self, uuid: Uuid) {
        *self.start_result.lock().unwrap() = Some(Ok(uuid));
    }

    pub fn expect_start_fails(&self, error: &str) {
        *self.start_result.lock().unwrap() = Some(Err(error.to_string()));
    }

    pub fn was_called(&self, method: &str) -> bool {
        self.call_log.lock().unwrap().iter().any(|m| m == method)
    }

    pub fn call_count(&self, method: &str) -> usize {
        self.call_log
            .lock()
            .unwrap()
            .iter()
            .filter(|m| *m == method)
            .count()
    }

    fn log_call(&self, method: &str) {
        self.call_log.lock().unwrap().push(method.to_string());
    }
}

// TODO: Implement DoraClient trait for MockDoraClient
// impl DoraClient for MockDoraClient { ... }

// ============================================================================
// MockLlmClient
// ============================================================================

/// Mock implementation of LlmClient for testing AI agent
pub struct MockLlmClient {
    responses: Arc<Mutex<VecDeque<AgentResponse>>>,
    received_messages: Arc<Mutex<Vec<String>>>,
}

impl MockLlmClient {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(VecDeque::new())),
            received_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_response(&self, response: AgentResponse) {
        self.responses.lock().unwrap().push_back(response);
    }

    pub fn set_continuation(&self, response: AgentResponse) {
        self.responses.lock().unwrap().push_back(response);
    }

    pub fn received_messages(&self) -> Vec<String> {
        self.received_messages.lock().unwrap().clone()
    }
}

// TODO: Implement LlmClient trait for MockLlmClient
// #[async_trait]
// impl LlmClient for MockLlmClient { ... }

// ============================================================================
// MockStorage
// ============================================================================

/// Mock implementation of Storage for testing
pub struct MockStorage {
    metrics: Arc<Mutex<Vec<NodeMetrics>>>,
    logs: Arc<Mutex<Vec<LogMessage>>>,
    spans: Arc<Mutex<Vec<Span>>>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
            logs: Arc::new(Mutex::new(Vec::new())),
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn set_metrics(&self, metrics: Vec<NodeMetrics>) {
        *self.metrics.lock().unwrap() = metrics;
    }

    pub fn set_logs(&self, logs: Vec<LogMessage>) {
        *self.logs.lock().unwrap() = logs;
    }

    pub fn set_spans(&self, spans: Vec<Span>) {
        *self.spans.lock().unwrap() = spans;
    }

    pub fn get_metrics(&self) -> Vec<NodeMetrics> {
        self.metrics.lock().unwrap().clone()
    }

    pub fn get_logs(&self) -> Vec<LogMessage> {
        self.logs.lock().unwrap().clone()
    }
}

// TODO: Implement Storage trait for MockStorage
// impl Storage for MockStorage { ... }

// ============================================================================
// MockCoordinator
// ============================================================================

/// Mock TCP server that simulates Dora Coordinator
pub struct MockCoordinator {
    addr: std::net::SocketAddr,
    // TODO: Add internal state and control methods
}

impl MockCoordinator {
    pub async fn start() -> Self {
        // TODO: Start mock TCP server
        todo!("Implement MockCoordinator::start")
    }

    pub fn addr(&self) -> String {
        self.addr.to_string()
    }

    pub async fn shutdown(&self) {
        // TODO: Shutdown mock server
        todo!("Implement MockCoordinator::shutdown")
    }

    pub fn set_response(&self, _response: &[u8]) {
        // TODO: Set response for next request
        todo!("Implement MockCoordinator::set_response")
    }

    pub fn emit_log(&self, _log: LogMessage) {
        // TODO: Emit log to subscribers
        todo!("Implement MockCoordinator::emit_log")
    }
}

// ============================================================================
// Placeholder types (to be imported from actual crates)
// ============================================================================

// TODO: Remove these when actual types are available
#[derive(Clone, Debug)]
pub struct DataflowEntry {
    pub uuid: Uuid,
    pub name: Option<String>,
    pub status: DataflowStatus,
    pub node_count: usize,
}

#[derive(Clone, Debug)]
pub enum DataflowStatus {
    Running,
    Finished,
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct NodeMetrics {
    pub node_id: String,
    pub cpu_percent: f32,
    pub memory_mb: f64,
}

#[derive(Clone, Debug)]
pub struct LogMessage {
    pub level: LogLevel,
    pub node_id: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub operation_name: String,
}

#[derive(Clone, Debug)]
pub enum AgentResponse {
    Text(String),
    ToolCall(ToolCall),
}

#[derive(Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

pub type Uuid = uuid::Uuid;
