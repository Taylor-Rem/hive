use std::collections::HashMap;
use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use crate::traits::{Agent, Worker, WorkerFactory, Tool, ToolFunction};
use crate::Message;

pub struct Queen {
    workers: HashMap<&'static str, Box<dyn Worker + Send + Sync>>
}

impl Agent for Queen {
    fn ollama_url(&self) -> &'static str { "http://localhost:11435/api/chat"  /* P40 (GPU 1) */ }
    fn model(&self) -> &'static str { "qwen2.5:32b-instruct-q5_K_M" }
    fn client(&self) -> Client { Client::new() }
    fn _type(&self) -> &'static str { "advanced" }
    fn system_prompt(&self) -> &'static str { SYSTEM_PROMPT_TEMPLATE }

    fn custom_placeholders(&self) -> Vec<(&'static str, String)> {
        let dir = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| ".".to_string());
        vec![
            ("{DIRECTORY}", dir),
        ]
    }
}

impl Queen {
    pub fn new() -> Queen {
        let workers = inventory::iter::<WorkerFactory>
            .into_iter()
            .map(|factory| {
                let worker = (factory.0)();
                (worker.role(), worker)
            })
            .collect();

        Queen { workers }
    }

    /// Build the list of available workers as a formatted string
    fn get_worker_list(&self) -> String {
        self.workers
            .values()
            .map(|w| format!("- **{}** [{}]: {}", w.role(), w.worker_type(), w.description()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Build the full system prompt with the worker list and placeholders
    pub fn build_system_prompt(&self) -> String {
        let mut prompt = SYSTEM_PROMPT_TEMPLATE.replace("{WORKERS}", &self.get_worker_list());

        // Apply custom placeholders
        for (placeholder, value) in Agent::custom_placeholders(self) {
            prompt = prompt.replace(placeholder, &value);
        }

        prompt
    }

    /// Build the delegate_to_worker tool with available worker names
    fn get_tools(&self) -> Vec<Tool> {
        let worker_names: Vec<&str> = self.workers.keys().copied().collect();

        vec![Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "delegate_to_worker".to_string(),
                description: "Delegate a task to a specialized worker".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "worker": {
                            "type": "string",
                            "enum": worker_names,
                            "description": "The worker to delegate to"
                        },
                        "instruction": {
                            "type": "string",
                            "description": "Natural language instruction for the worker"
                        }
                    },
                    "required": ["worker", "instruction"]
                }),
            },
        }]
    }

    /// Execute a tool call and return the result
    async fn execute_tool_call(&self, name: &str, arguments: &serde_json::Value) -> Result<String> {
        match name {
            "delegate_to_worker" => {
                let worker_name = arguments["worker"].as_str().unwrap_or("");
                let instruction = arguments["instruction"].as_str().unwrap_or("");

                eprintln!("[QUEEN] Delegating to worker '{}' with instruction: {}", worker_name, instruction);

                if let Some(worker) = self.workers.get(worker_name) {
                    worker.process(instruction).await
                } else {
                    eprintln!("[QUEEN] Error: Worker '{}' not found", worker_name);
                    Ok(format!("Error: Worker '{}' not found", worker_name))
                }
            }
            _ => {
                eprintln!("[QUEEN] Error: Unknown tool '{}'", name);
                Ok(format!("Error: Unknown tool '{}'", name))
            }
        }
    }

    /// Run the agentic loop until we get a final response
    pub async fn run_agentic_loop(&self, messages: &mut Vec<Message>) -> Result<String> {
        let tools = self.get_tools();
        let worker_names: Vec<&str> = self.workers.keys().copied().collect();

        eprintln!("[QUEEN] === Starting Queen's Agentic Loop ===");
        eprintln!("[QUEEN] Available workers: {:?}", worker_names);

        let mut iteration = 0;
        loop {
            iteration += 1;
            eprintln!("[QUEEN] --- Iteration {} ---", iteration);

            // Make request with tools
            let response = self.make_request(messages, Some(tools.clone())).await?;

            // Add response to message history
            messages.push(response.clone());

            // Check if there are tool calls to process
            if let Some(tool_calls) = &response.tool_calls {
                eprintln!("[QUEEN] Received {} tool call(s)", tool_calls.len());

                for tool_call in tool_calls {
                    let name = &tool_call.function.name;
                    let arguments = &tool_call.function.arguments;

                    eprintln!("[QUEEN] Tool call: {}({})", name, arguments);

                    let result = self.execute_tool_call(name, arguments).await?;

                    // Add tool result to messages
                    messages.push(Message {
                        role: "tool".to_string(),
                        content: Some(result),
                        tool_calls: None,
                    });
                }
            } else {
                // No tool calls - we have the final response
                let final_response = response.content.unwrap_or_default();
                return Ok(final_response);
            }
        }
    }
}

const SYSTEM_PROMPT_TEMPLATE: &str = r#"You are the Queen of Hive. You are the ONLY agent that communicates with the user.

# Environment
- Operating System: Linux
- Current Directory: {DIRECTORY}
- Workers share this directory. Use RELATIVE paths (e.g., "src/main.rs", "." for current dir) not absolute paths.

# Core Architecture
- YOU talk to the user. Workers NEVER talk to the user.
- Workers are TOOLS, not assistants. They execute operations and return raw data.
- YOU do all thinking, analysis, and synthesis. Workers just fetch and execute.

# Worker Types
Workers are categorized by complexity:

**simple**: Low-capability workers using small models. Give them atomic, single-operation instructions. They cannot reason, plan, or handle ambiguity. Be explicit and precise.
- Example: "Read the file at 'src/main.rs'" (not "Look at main.rs and tell me what's important")

**advanced**: High-capability workers using larger models. They can handle complex, multi-step tasks and understand context. Provide full context and let them determine the approach.
- Example: "Refactor this function to use async/await, ensuring error handling is preserved"

# How to Use Workers
For **simple** workers: Give specific, atomic operations. They return raw results - YOU do the analysis.

WRONG: "Give me an overview of the project directory"
RIGHT: "List the directory at '.'" → then YOU analyze the listing

WRONG: "Explain what's in this file"
RIGHT: "Read the file at 'src/main.rs'" → then YOU explain it

For **advanced** workers: Provide full context and goals. They can handle reasoning and multi-step tasks.

# Available Workers
{WORKERS}

# Your Workflow
1. Receive user request
2. Break it down into operations appropriate for each worker's type
3. For simple workers: request raw data one operation at a time
4. For advanced workers: provide full context and let them work
5. Analyze and synthesize results yourself
6. Respond to the user with your analysis

# Example
User: "What does this project do?"
You should:
1. delegate_to_worker("file_manager", "List directory at '.'")
2. delegate_to_worker("file_manager", "Read file 'Cargo.toml'")
3. delegate_to_worker("file_manager", "Read file 'src/main.rs'")
4. Analyze all the raw data yourself
5. Give the user YOUR summary

# Communication
- Be direct and helpful to the user
- Do your own thinking - don't outsource analysis to workers
- If you need more data, request it from workers
- Workers return data; you return answers

# Remember
- You have full access to read the user's directory and files
"#;