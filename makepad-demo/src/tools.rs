use serde::Serialize;
use std::process::Command;

/// Tool definition for Claude API
#[derive(Debug, Clone, Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Result of executing a tool
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub tool_use_id: String,
    pub content: String,
    pub is_error: bool,
}

/// Get all available dora tools for Claude
pub fn get_dora_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "dora_list".to_string(),
            description: "List all running dataflows. Returns information about active dora dataflows including their IDs and status.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDefinition {
            name: "dora_start".to_string(),
            description: "Start a new dataflow from a YAML file. The dataflow_path should be the path to a valid dora dataflow YAML configuration file.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "dataflow_path": {
                        "type": "string",
                        "description": "Path to the dataflow YAML file to start"
                    }
                },
                "required": ["dataflow_path"]
            }),
        },
        ToolDefinition {
            name: "dora_stop".to_string(),
            description: "Stop a running dataflow by its UUID or name.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "dataflow_id": {
                        "type": "string",
                        "description": "UUID or name of the dataflow to stop"
                    }
                },
                "required": ["dataflow_id"]
            }),
        },
        ToolDefinition {
            name: "dora_destroy".to_string(),
            description: "Destroy (forcefully stop) a dataflow and clean up all resources.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "dataflow_id": {
                        "type": "string",
                        "description": "UUID or name of the dataflow to destroy"
                    }
                },
                "required": ["dataflow_id"]
            }),
        },
        ToolDefinition {
            name: "dora_logs".to_string(),
            description: "Get logs from a running dataflow. Optionally filter by node name.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "dataflow_id": {
                        "type": "string",
                        "description": "UUID or name of the dataflow"
                    },
                    "node": {
                        "type": "string",
                        "description": "Optional: filter logs by node name"
                    }
                },
                "required": ["dataflow_id"]
            }),
        },
        ToolDefinition {
            name: "shell_command".to_string(),
            description: "Execute a shell command. Use this for general system commands, file operations, or when dora-specific commands are not sufficient. Be careful with this tool.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    },
                    "working_dir": {
                        "type": "string",
                        "description": "Optional: working directory for the command"
                    }
                },
                "required": ["command"]
            }),
        },
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file. Useful for inspecting dataflow YAML files or checking configurations.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file. Useful for creating or modifying dataflow YAML files.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
        },
        ToolDefinition {
            name: "list_directory".to_string(),
            description: "List files and directories in a given path.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path to list"
                    }
                },
                "required": ["path"]
            }),
        },
    ]
}

/// Execute a tool by name with given arguments
pub fn execute_tool(name: &str, tool_use_id: &str, args: &serde_json::Value) -> ToolResult {
    let result = match name {
        "dora_list" => execute_dora_list(),
        "dora_start" => execute_dora_start(args),
        "dora_stop" => execute_dora_stop(args),
        "dora_destroy" => execute_dora_destroy(args),
        "dora_logs" => execute_dora_logs(args),
        "shell_command" => execute_shell_command(args),
        "read_file" => execute_read_file(args),
        "write_file" => execute_write_file(args),
        "list_directory" => execute_list_directory(args),
        _ => Err(format!("Unknown tool: {}", name)),
    };

    match result {
        Ok(content) => ToolResult {
            tool_use_id: tool_use_id.to_string(),
            content,
            is_error: false,
        },
        Err(error) => ToolResult {
            tool_use_id: tool_use_id.to_string(),
            content: error,
            is_error: true,
        },
    }
}

fn execute_dora_list() -> Result<String, String> {
    run_command("dora", &["list", "--format", "json"])
}

fn execute_dora_start(args: &serde_json::Value) -> Result<String, String> {
    let path = args
        .get("dataflow_path")
        .and_then(|v| v.as_str())
        .ok_or("Missing dataflow_path argument")?;

    run_command("dora", &["start", "--detach", path])
}

fn execute_dora_stop(args: &serde_json::Value) -> Result<String, String> {
    let id = args
        .get("dataflow_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing dataflow_id argument")?;

    run_command("dora", &["stop", id])
}

fn execute_dora_destroy(args: &serde_json::Value) -> Result<String, String> {
    let id = args
        .get("dataflow_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing dataflow_id argument")?;

    run_command("dora", &["destroy", id])
}

fn execute_dora_logs(args: &serde_json::Value) -> Result<String, String> {
    let id = args
        .get("dataflow_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing dataflow_id argument")?;

    let mut cmd_args = vec!["logs", id];

    if let Some(node) = args.get("node").and_then(|v| v.as_str()) {
        cmd_args.push("--node");
        cmd_args.push(node);
    }

    run_command("dora", &cmd_args)
}

fn execute_shell_command(args: &serde_json::Value) -> Result<String, String> {
    let command = args
        .get("command")
        .and_then(|v| v.as_str())
        .ok_or("Missing command argument")?;

    let working_dir = args.get("working_dir").and_then(|v| v.as_str());

    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", command]);
        c
    } else {
        let mut c = Command::new("sh");
        c.args(["-c", command]);
        c
    };

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(format!("{}{}", stdout, stderr))
    } else {
        Err(format!(
            "Command failed with exit code {:?}\nstdout: {}\nstderr: {}",
            output.status.code(),
            stdout,
            stderr
        ))
    }
}

fn execute_read_file(args: &serde_json::Value) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or("Missing path argument")?;

    std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
}

fn execute_write_file(args: &serde_json::Value) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or("Missing path argument")?;

    let content = args
        .get("content")
        .and_then(|v| v.as_str())
        .ok_or("Missing content argument")?;

    std::fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))?;

    Ok(format!(
        "Successfully wrote {} bytes to {}",
        content.len(),
        path
    ))
}

fn execute_list_directory(args: &serde_json::Value) -> Result<String, String> {
    let path = args
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or("Missing path argument")?;

    let entries =
        std::fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut result = Vec::new();
    for entry in entries.flatten() {
        let file_type = entry.file_type().ok();
        let type_str = match file_type {
            Some(ft) if ft.is_dir() => "[DIR]",
            Some(ft) if ft.is_file() => "[FILE]",
            Some(ft) if ft.is_symlink() => "[LINK]",
            _ => "[?]",
        };
        result.push(format!(
            "{} {}",
            type_str,
            entry.file_name().to_string_lossy()
        ));
    }

    Ok(result.join("\n"))
}

fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute {}: {}", program, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        Ok(format!("{}{}", stdout, stderr))
    } else {
        Err(format!(
            "{} failed with exit code {:?}\nstdout: {}\nstderr: {}",
            program,
            output.status.code(),
            stdout,
            stderr
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_dora_tools() {
        let tools = get_dora_tools();
        assert!(!tools.is_empty());

        // Check that essential tools are present
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"dora_list"));
        assert!(tool_names.contains(&"dora_start"));
        assert!(tool_names.contains(&"dora_stop"));
        assert!(tool_names.contains(&"shell_command"));
        assert!(tool_names.contains(&"read_file"));
    }

    #[test]
    fn test_tool_definition_serialization() {
        let tool = ToolDefinition {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        };

        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("test_tool"));
        assert!(json.contains("A test tool"));
    }

    #[test]
    fn test_execute_unknown_tool() {
        let result = execute_tool("unknown_tool", "test-id", &serde_json::json!({}));
        assert!(result.is_error);
        assert!(result.content.contains("Unknown tool"));
    }

    #[test]
    fn test_execute_list_directory() {
        let args = serde_json::json!({ "path": "." });
        let result = execute_tool("list_directory", "test-id", &args);
        // Should succeed for current directory
        assert!(!result.is_error);
    }

    #[test]
    fn test_execute_read_file_missing_arg() {
        let result = execute_tool("read_file", "test-id", &serde_json::json!({}));
        assert!(result.is_error);
        assert!(result.content.contains("Missing path"));
    }

    #[test]
    fn test_tool_result_structure() {
        let result = ToolResult {
            tool_use_id: "123".to_string(),
            content: "test content".to_string(),
            is_error: false,
        };
        assert_eq!(result.tool_use_id, "123");
        assert_eq!(result.content, "test content");
        assert!(!result.is_error);
    }
}
