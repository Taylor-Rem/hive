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

    // fn description(&self) -> &'static str {
    //     "Manages file system operations including reading, writing, and organizing files"
    // }

    async fn process(&self, instruction: &str) -> Result<String> {
        // Delegate to Agent's run method
        Agent::run(self, instruction).await
    }
}

impl Agent for FileManager {
    fn ollama_url(&self) -> &'static str {
        "http://localhost:11434/api/chat"  // RTX 3070 (GPU 0)
    }

    fn model(&self) -> &'static str {
        "qwen2.5:7b"
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

const SYSTEM_PROMPT: &str = r#"You are a file operation executor. You receive commands and execute them using your tools.

# Rules
1. Parse the instruction to identify the operation and path
2. Call the appropriate tool
3. Return ONLY the raw result - no commentary, no analysis, no explanation

# Operations
- "List directory at X" or "List X" → call list_directory with path X
- "Read file X" or "Read X" → call read_file with path X
- "Write to X" with content → call write_file
- "Delete X" → call delete_file
- "Create directory X" → call create_directory

# Response Format
Return the tool result directly. Do not add any interpretation or suggestions.

Example:
Instruction: "List directory at '.'"
Action: Call list_directory(path=".")
Response: ["Cargo.toml", "src", ".gitignore"]

That's it. Just the data."#;
