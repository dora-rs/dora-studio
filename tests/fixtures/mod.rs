//! Test fixtures and factory functions
//!
//! Provides helper functions to create test data consistently.

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

// Re-export mock types for convenience
pub use super::mocks::*;

// ============================================================================
// Dataflow Fixtures
// ============================================================================

/// Create a mock dataflow entry with sensible defaults
pub fn mock_dataflow(name: &str) -> DataflowEntry {
    DataflowEntry {
        uuid: Uuid::new_v4(),
        name: Some(name.to_string()),
        status: DataflowStatus::Running,
        node_count: 4,
    }
}

/// Create a mock dataflow with specific status
pub fn mock_dataflow_with_status(name: &str, status: DataflowStatus) -> DataflowEntry {
    DataflowEntry {
        uuid: Uuid::new_v4(),
        name: Some(name.to_string()),
        status,
        node_count: 4,
    }
}

/// Create a mock dataflow with specific UUID
pub fn mock_dataflow_with_uuid(name: &str, uuid: Uuid) -> DataflowEntry {
    DataflowEntry {
        uuid,
        name: Some(name.to_string()),
        status: DataflowStatus::Running,
        node_count: 4,
    }
}

// ============================================================================
// Log Fixtures
// ============================================================================

/// Create a log entry with default timestamp
pub fn log_entry(level: LogLevel, message: &str) -> LogMessage {
    LogMessage {
        level,
        node_id: None,
        message: message.to_string(),
    }
}

/// Create a log entry with specific node
pub fn log_entry_node(node_id: &str, message: &str) -> LogMessage {
    LogMessage {
        level: LogLevel::Info,
        node_id: Some(node_id.to_string()),
        message: message.to_string(),
    }
}

/// Generate sample logs for testing
pub fn generate_sample_logs(count: usize) -> Vec<LogMessage> {
    (0..count)
        .map(|i| {
            let level = match i % 4 {
                0 => LogLevel::Debug,
                1 => LogLevel::Info,
                2 => LogLevel::Warn,
                _ => LogLevel::Error,
            };
            LogMessage {
                level,
                node_id: Some(format!("node-{}", i % 5)),
                message: format!("Sample log message {}", i),
            }
        })
        .collect()
}

// ============================================================================
// Metrics Fixtures
// ============================================================================

/// Create a node metric entry
pub fn node_metric(node_id: &str, cpu: f32, memory: f64) -> NodeMetrics {
    NodeMetrics {
        node_id: node_id.to_string(),
        cpu_percent: cpu,
        memory_mb: memory,
    }
}

/// Generate sample metrics for multiple nodes
pub fn generate_node_metrics(nodes: &[&str]) -> Vec<NodeMetrics> {
    nodes
        .iter()
        .enumerate()
        .map(|(i, node_id)| NodeMetrics {
            node_id: node_id.to_string(),
            cpu_percent: (i as f32 * 20.0) % 100.0,
            memory_mb: (i as f64 * 256.0) % 4096.0,
        })
        .collect()
}

// ============================================================================
// Span/Trace Fixtures
// ============================================================================

/// Create a span entry
pub fn span_entry(trace_id: &str, operation: &str, duration_ms: i64) -> Span {
    Span {
        trace_id: trace_id.to_string(),
        span_id: Uuid::new_v4().to_string(),
        operation_name: operation.to_string(),
    }
}

/// Generate a trace with multiple spans
pub fn generate_trace(trace_id: &str, operations: &[&str]) -> Vec<Span> {
    operations
        .iter()
        .map(|op| Span {
            trace_id: trace_id.to_string(),
            span_id: Uuid::new_v4().to_string(),
            operation_name: op.to_string(),
        })
        .collect()
}

// ============================================================================
// YAML Fixtures
// ============================================================================

/// Simple valid dataflow YAML
pub fn simple_dataflow_yaml() -> &'static str {
    r#"
nodes:
  - id: camera
    path: dora-webcam
    outputs: [image]
"#
}

/// Dataflow YAML with connections
pub fn connected_dataflow_yaml() -> &'static str {
    r#"
nodes:
  - id: camera
    path: dora-webcam
    outputs: [image]
  - id: detector
    path: dora-yolo
    inputs:
      image: camera/image
    outputs: [bbox]
  - id: plot
    path: dora-plot
    inputs:
      image: camera/image
      bbox: detector/bbox
"#
}

/// Invalid YAML for testing error handling
pub fn invalid_yaml() -> &'static str {
    "invalid: yaml: syntax::: broken"
}

/// YAML with missing required field (path)
pub fn yaml_missing_path() -> &'static str {
    r#"
nodes:
  - id: camera
    outputs: [image]
"#
}

/// YAML with duplicate node IDs
pub fn yaml_duplicate_ids() -> &'static str {
    r#"
nodes:
  - id: node1
    path: test-path
  - id: node1
    path: test-path-2
"#
}

/// Generate large dataflow YAML for performance testing
pub fn generate_large_dataflow_yaml(node_count: usize) -> String {
    let mut yaml = String::from("nodes:\n");
    for i in 0..node_count {
        yaml.push_str(&format!(
            "  - id: node-{}\n    path: test-node\n    outputs: [out]\n",
            i
        ));
        if i > 0 {
            yaml.push_str(&format!("    inputs:\n      in: node-{}/out\n", i - 1));
        }
    }
    yaml
}

// ============================================================================
// AI/Tool Fixtures
// ============================================================================

/// Create a tool call fixture
pub fn tool_call(name: &str, args: serde_json::Value) -> ToolCall {
    ToolCall {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        arguments: args,
    }
}

/// Create a text response
pub fn text_response(text: &str) -> AgentResponse {
    AgentResponse::Text(text.to_string())
}

/// Create a tool call response
pub fn tool_call_response(name: &str, args: serde_json::Value) -> AgentResponse {
    AgentResponse::ToolCall(tool_call(name, args))
}
