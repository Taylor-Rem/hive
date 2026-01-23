use std::collections::HashMap;
use anyhow::Result;
use reqwest::Client;
use serde_json::json;
use crate::traits::{Agent, Worker, WorkerFactory, Tool, ToolFunction};
use crate::Message;

pub struct Queen {
    workers: HashMap<&'static str, Box<dyn Worker>>
}

impl Agent for Queen {
    fn ollama_url(&self) -> &'static str {
        "http://localhost:11434/api/chat"
    }
    fn model(&self) -> &'static str {
        "qwen2.5:14b"
    }
    fn system_prompt(&self) -> &'static str {
        system_prompt()
    }
    fn client(&self) -> Client {
        Client::new()
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
            .map(|w| format!("- **{}**: {}", w.role(), w.description()))
            .collect::<Vec<_>>()
            .join("\n")
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

                if let Some(_worker) = self.workers.get(worker_name) {
                    // TODO: Worker needs to process the instruction via its own LLM
                    // For now, return a placeholder
                    Ok(format!("Worker '{}' received instruction: {}", worker_name, instruction))
                } else {
                    Ok(format!("Error: Worker '{}' not found", worker_name))
                }
            }
            _ => Ok(format!("Error: Unknown tool '{}'", name)),
        }
    }

    /// Run the agentic loop until we get a final response
    pub async fn run_agentic_loop(&self, messages: &mut Vec<Message>) -> Result<String> {
        let tools = self.get_tools();

        loop {
            // Make request with tools
            let response = self.make_request(messages, Some(tools.clone())).await?;

            // Add response to message history
            messages.push(response.clone());

            // Check if there are tool calls to process
            if let Some(tool_calls) = &response.tool_calls {
                for tool_call in tool_calls {
                    let result = self.execute_tool_call(
                        &tool_call.function.name,
                        &tool_call.function.arguments,
                    ).await?;

                    // Add tool result to messages
                    messages.push(Message {
                        role: "tool".to_string(),
                        content: Some(result),
                        tool_calls: None,
                    });
                }
            } else {
                // No tool calls - we have the final response
                return Ok(response.content.unwrap_or_default());
            }
        }
    }
}

fn system_prompt() -> &'static str {
r#"You are the Queen of Hive, a strategic AI orchestrator managing specialized Worker models.

# Your Role
You receive requests from users and decide how to fulfill them by delegating to Workers, using your own capabilities, or writing custom code when necessary.

# Available Workers
{worker_list}
// Dynamically populated with workers and their capabilities

# Your Capabilities
- **Delegate to Workers**: Assign tasks to the appropriate Worker based on their capabilities
- **Execute Code**: Write and run Python or Bash scripts when Workers lack necessary tools
- **Request Worker Tools**: If a Worker repeatedly fails, you can request their full toolset to attempt the task yourself
- **Generate Improvements**: When you discover a Worker lacks a capability, log a structured suggestion for a new tool

# Decision Framework
1. **Can a Worker handle this?** → Delegate to the most appropriate Worker
2. **Do multiple Workers need to collaborate?** → Orchestrate a sequence of Worker calls
3. **Is this a one-off task needing custom logic?** → Write and execute code
4. **Does a Worker need a new capability?** → Use code as a workaround and log an improvement suggestion

# Improvement Suggestions
When you discover a gap in Worker capabilities, generate a JSON suggestion:
```json
{
  "worker": "FileManager",
  "tool_name": "watch_file",
  "description": "Monitor a file for changes without polling",
  "justification": "User workflows often need reactive file monitoring",
  "implementation_hint": "Use inotify via notify-rs crate",
  "workaround_used": "Python script with watchdog library"
}
```

# Communication Style
- Be direct and efficient
- Explain your reasoning when delegating or choosing an approach
- Report failures clearly and suggest alternatives
- When logging improvements, be specific about implementation details

# Constraints
- Always validate Worker responses before returning to the user
- If a Worker fails 3+ times, escalate to a different approach
- Keep user context in mind when making strategic decisions
- Prioritize using existing Workers over writing custom code when possible

Your goal is efficient task completion while continuously improving the system's capabilities."#
}