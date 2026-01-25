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
    directory: PathBuf,
}

#[async_trait]
impl Worker for FileManager {
    fn role(&self) -> &'static str {
        "file_manager"
    }

    fn description(&self) -> &'static str {
        "Manages file system operations including reading, writing, and organizing files"
    }

    fn worker_type(&self) -> &'static str {
        "simple"
    }

    async fn process(&self, instruction: &str) -> Result<String> {
        // Delegate to Agent's run method
        Agent::run(self, instruction).await
    }
}

impl Agent for FileManager {
    fn ollama_url(&self) -> &'static str { "http://localhost:11434/api/chat"  /* RTX 3070 (GPU 0) */ }
    fn model(&self) -> &'static str { "qwen2.5:7b" }
    fn client(&self) -> Client { Client::new() }
    fn _type(&self) -> &'static str { "simple" }
    fn system_prompt(&self) -> &'static str { SYSTEM_PROMPT }

    fn custom_placeholders(&self) -> Vec<(&'static str, String)> {
        vec![
            ("{DIRECTORY}", self.directory.display().to_string()),
        ]
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
            Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "append_file".to_string(),
                    description: "Append content to the end of a file".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the file to append to"
                            },
                            "content": {
                                "type": "string",
                                "description": "Content to append to the file"
                            }
                        },
                        "required": ["path", "content"]
                    }),
                },
            },
            Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "insert_at_line".to_string(),
                    description: "Insert content at a specific line number (1-indexed)".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the file"
                            },
                            "line_number": {
                                "type": "integer",
                                "description": "Line number to insert at (1-indexed, content inserted before this line)"
                            },
                            "content": {
                                "type": "string",
                                "description": "Content to insert"
                            }
                        },
                        "required": ["path", "line_number", "content"]
                    }),
                },
            },
            Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "replace_text".to_string(),
                    description: "Find and replace text in a file".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the file"
                            },
                            "old_text": {
                                "type": "string",
                                "description": "Text to find"
                            },
                            "new_text": {
                                "type": "string",
                                "description": "Text to replace it with"
                            }
                        },
                        "required": ["path", "old_text", "new_text"]
                    }),
                },
            },
        ]
    }

    fn execute_tool(&self, name: &str, args: &serde_json::Value) -> Result<String> {
        match name {
            "read_file" => {
                let path = args["path"].as_str().unwrap_or("");
                let full_path = self.directory.join(path);
                match fs::read_to_string(&full_path) {
                    Ok(content) => Ok(content),
                    Err(e) => Ok(format!("Error reading file: {}", e)),
                }
            }
            "write_file" => {
                let path = args["path"].as_str().unwrap_or("");
                let content = args["content"].as_str().unwrap_or("");
                let full_path = self.directory.join(path);
                match fs::write(&full_path, content) {
                    Ok(_) => Ok(format!("Successfully wrote to {}", path)),
                    Err(e) => Ok(format!("Error writing file: {}", e)),
                }
            }
            "list_directory" => {
                let path = args["path"].as_str().unwrap_or(".");
                let full_path = self.directory.join(path);
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
                let full_path = self.directory.join(path);
                match fs::remove_file(&full_path) {
                    Ok(_) => Ok(format!("Successfully deleted {}", path)),
                    Err(e) => Ok(format!("Error deleting file: {}", e)),
                }
            }
            "create_directory" => {
                let path = args["path"].as_str().unwrap_or("");
                let full_path = self.directory.join(path);
                match fs::create_dir_all(&full_path) {
                    Ok(_) => Ok(format!("Successfully created directory {}", path)),
                    Err(e) => Ok(format!("Error creating directory: {}", e)),
                }
            }
            "append_file" => {
                let path = args["path"].as_str().unwrap_or("");
                let content = args["content"].as_str().unwrap_or("");
                let full_path = self.directory.join(path);

                use std::fs::OpenOptions;
                use std::io::Write;

                match OpenOptions::new().append(true).create(true).open(&full_path) {
                    Ok(mut file) => {
                        match file.write_all(content.as_bytes()) {
                            Ok(_) => Ok(format!("Successfully appended to {}", path)),
                            Err(e) => Ok(format!("Error writing to file: {}", e)),
                        }
                    }
                    Err(e) => Ok(format!("Error opening file: {}", e)),
                }
            }
            "insert_at_line" => {
                let path = args["path"].as_str().unwrap_or("");
                let line_number = args["line_number"].as_u64().unwrap_or(1) as usize;
                let content = args["content"].as_str().unwrap_or("");
                let full_path = self.directory.join(path);

                match fs::read_to_string(&full_path) {
                    Ok(file_content) => {
                        let mut lines: Vec<&str> = file_content.lines().collect();
                        let insert_idx = if line_number == 0 { 0 } else { (line_number - 1).min(lines.len()) };

                        // Insert the new content as a line
                        lines.insert(insert_idx, content);

                        let new_content = lines.join("\n");
                        match fs::write(&full_path, new_content) {
                            Ok(_) => Ok(format!("Successfully inserted at line {} in {}", line_number, path)),
                            Err(e) => Ok(format!("Error writing file: {}", e)),
                        }
                    }
                    Err(e) => Ok(format!("Error reading file: {}", e)),
                }
            }
            "replace_text" => {
                let path = args["path"].as_str().unwrap_or("");
                let old_text = args["old_text"].as_str().unwrap_or("");
                let new_text = args["new_text"].as_str().unwrap_or("");
                let full_path = self.directory.join(path);

                match fs::read_to_string(&full_path) {
                    Ok(file_content) => {
                        if !file_content.contains(old_text) {
                            return Ok(format!("Error: old_text not found in {}", path));
                        }

                        let new_content = file_content.replacen(old_text, new_text, 1);
                        match fs::write(&full_path, new_content) {
                            Ok(_) => Ok(format!("Successfully replaced text in {}", path)),
                            Err(e) => Ok(format!("Error writing file: {}", e)),
                        }
                    }
                    Err(e) => Ok(format!("Error reading file: {}", e)),
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
            None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        };
        FileManager { directory: base }
    }
}

const SYSTEM_PROMPT: &str = r#"You are a file operation executor. You receive commands and execute them using your tools.

# Rules
1. Parse the instruction to identify the operation and path
2. Call the appropriate tool
3. Return ONLY the raw result - no commentary, no analysis, no explanation

# Current Directory
{DIRECTORY}

# Available Tools
{TOOLS}

# Response Format
Return the tool result directly. Do not add any interpretation or suggestions.
Just the data.

# Important
Do not create files or directories unless explicitely asked.
If you are asked to find a directory, you do not fulfill the request by creating a directory then returning those results.
"#;
