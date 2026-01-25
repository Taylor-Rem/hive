use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::process::Command;
use crate::traits::{Worker, WorkerFactory, Agent, Tool, ToolFunction};

inventory::submit! {
    WorkerFactory(|| Box::new(Shell::new()))
}

pub struct Shell {
    working_dir: std::path::PathBuf,
}

#[async_trait]
impl Worker for Shell {
    fn role(&self) -> &'static str {
        "shell"
    }

    fn description(&self) -> &'static str {
        "Executes command line operations. Can run shell commands and return their output."
    }

    fn worker_type(&self) -> &'static str {
        "simple"
    }

    async fn process(&self, instruction: &str) -> Result<String> {
        Agent::run(self, instruction).await
    }
}

impl Agent for Shell {
    fn ollama_url(&self) -> &'static str { "http://localhost:11434/api/chat" /* RTX 3070 (GPU 0) */ }
    fn model(&self) -> &'static str { "qwen2.5:7b" }
    fn client(&self) -> Client { Client::new() }
    fn _type(&self) -> &'static str { "simple" }
    fn system_prompt(&self) -> &'static str { SYSTEM_PROMPT }

    fn custom_placeholders(&self) -> Vec<(&'static str, String)> {
        vec![
            ("{DIRECTORY}", self.working_dir.display().to_string()),
        ]
    }

    fn get_tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: "execute_command".to_string(),
                    description: "Execute a shell command and return its output".to_string(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "command": {
                                "type": "string",
                                "description": "The command to execute"
                            }
                        },
                        "required": ["command"]
                    }),
                },
            },
        ]
    }

    fn execute_tool(&self, name: &str, args: &serde_json::Value) -> Result<String> {
        match name {
            "execute_command" => {
                let command = args["command"].as_str().unwrap_or("");

                let output = Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .current_dir(&self.working_dir)
                    .output();

                match output {
                    Ok(result) => {
                        let stdout = String::from_utf8_lossy(&result.stdout);
                        let stderr = String::from_utf8_lossy(&result.stderr);

                        if result.status.success() {
                            if stdout.is_empty() {
                                Ok("Command executed successfully (no output)".to_string())
                            } else {
                                Ok(stdout.to_string())
                            }
                        } else {
                            Ok(format!("Command failed (exit code: {:?})\nstdout: {}\nstderr: {}",
                                result.status.code(), stdout, stderr))
                        }
                    }
                    Err(e) => Ok(format!("Error executing command: {}", e)),
                }
            }
            _ => Ok(format!("Unknown tool: {}", name)),
        }
    }
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            working_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        }
    }
}

const SYSTEM_PROMPT: &str = r#"You are a shell command executor. You receive instructions and execute shell commands.

# Working Directory
{DIRECTORY}

# Rules
1. Parse the instruction to identify what command to run
2. Execute the appropriate command
3. Return ONLY the raw output - no commentary

# Available Tools
{TOOLS}

# Response Format
Return the command output directly. Do not add interpretation or suggestions.
Just the data."#;
