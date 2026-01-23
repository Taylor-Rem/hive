use std::path::PathBuf;
use std::fs;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use crate::traits::{Worker, WorkerFactory, Agent, Tool, ToolFunction};

inventory::submit! {
    WorkerFactory(|| Box::new(FileManager::new(None)))
}

pub struct FileManager {
    base: PathBuf,
}

#[async_trait]
impl Worker for FileManager {
    fn role(&self) -> &'static str {
        "file_manager"
    }

    fn description(&self) -> &'static str {
        "Manages file system operations including reading, writing, and organizing files"
    }

    async fn process(&self, instruction: &str) -> Result<String> {
        // Delegate to Agent's run method
        Agent::run(self, instruction).await
    }
}

impl Agent for FileManager {
    fn ollama_url(&self) -> &'static str {
        "http://localhost:11434/api/chat"
    }

    fn model(&self) -> &'static str {
        "qwen2.5:14b"
    }

    fn system_prompt(&self) -> &'static str {
        SYSTEM_PROMPT
    }

    fn client(&self) -> Client {
        Client::new()
    }

    fn get_tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "read_file".to_string(),
                    description: "Read the contents of a file".to_string(),
                    parameters: json!({
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
            },
            Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "write_file".to_string(),
                    description: "Write content to a file (creates or overwrites)".to_string(),
                    parameters: json!({
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
            },
            Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "list_directory".to_string(),
                    description: "List files and directories in a path".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the directory to list"
                            }
                        },
                        "required": ["path"]
                    }),
                },
            },
            Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "delete_file".to_string(),
                    description: "Delete a file".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the file to delete"
                            }
                        },
                        "required": ["path"]
                    }),
                },
            },
            Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "create_directory".to_string(),
                    description: "Create a directory (and parent directories if needed)".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the directory to create"
                            }
                        },
                        "required": ["path"]
                    }),
                },
            },
        ]
    }

    fn execute_tool(&self, name: &str, args: &serde_json::Value) -> Result<String> {
        match name {
            "read_file" => {
                let path = args["path"].as_str().unwrap_or("");
                let full_path = self.base.join(path);
                match fs::read_to_string(&full_path) {
                    Ok(content) => Ok(content),
                    Err(e) => Ok(format!("Error reading file: {}", e)),
                }
            }
            "write_file" => {
                let path = args["path"].as_str().unwrap_or("");
                let content = args["content"].as_str().unwrap_or("");
                let full_path = self.base.join(path);
                match fs::write(&full_path, content) {
                    Ok(_) => Ok(format!("Successfully wrote to {}", path)),
                    Err(e) => Ok(format!("Error writing file: {}", e)),
                }
            }
            "list_directory" => {
                let path = args["path"].as_str().unwrap_or(".");
                let full_path = self.base.join(path);
                match fs::read_dir(&full_path) {
                    Ok(entries) => {
                        let files: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .map(|e| e.file_name().to_string_lossy().to_string())
                            .collect();
                        Ok(json!(files).to_string())
                    }
                    Err(e) => Ok(format!("Error listing directory: {}", e)),
                }
            }
            "delete_file" => {
                let path = args["path"].as_str().unwrap_or("");
                let full_path = self.base.join(path);
                match fs::remove_file(&full_path) {
                    Ok(_) => Ok(format!("Successfully deleted {}", path)),
                    Err(e) => Ok(format!("Error deleting file: {}", e)),
                }
            }
            "create_directory" => {
                let path = args["path"].as_str().unwrap_or("");
                let full_path = self.base.join(path);
                match fs::create_dir_all(&full_path) {
                    Ok(_) => Ok(format!("Successfully created directory {}", path)),
                    Err(e) => Ok(format!("Error creating directory: {}", e)),
                }
            }
            _ => Ok(format!("Unknown tool: {}", name)),
        }
    }
}

impl FileManager {
    pub fn new(path: Option<&str>) -> Self {
        let base = match path {
            Some(p) => PathBuf::from(p),
            None => PathBuf::from("."),
        };
        FileManager { base }
    }
}

const SYSTEM_PROMPT: &str = r#"You are FileManager, a specialized Worker in the Hive system focused on file operations.

# Your Role
You receive file-related tasks from the Queen and execute them using your available tools. You report results back to the Queen clearly and concisely.

# Operational Guidelines
- Validate file paths before operations
- Handle errors gracefully and report them clearly
- For complex tasks, break them into multiple tool calls
- Return structured data when possible (JSON for lists/objects)
- Be explicit about what succeeded vs. failed

# Response Format
When you have completed the task, provide a clear summary of what was done.

# Constraints
- Only operate on files you have permission to access
- Never execute file contents as code
- Stay focused on file operations

You are efficient, reliable, and clear about your capabilities and limitations."#;
