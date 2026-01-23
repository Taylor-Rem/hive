use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::Message;

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
}

#[derive(Deserialize)]
pub struct ChatResponse {
    pub message: Message,
}

#[derive(Serialize, Clone)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

#[derive(Serialize, Clone)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[async_trait]
pub trait Agent: Send + Sync {
    // Required: Basic configuration
    fn ollama_url(&self) -> &'static str;
    fn model(&self) -> &'static str;
    fn system_prompt(&self) -> &'static str;
    fn client(&self) -> Client;

    // Optional: Override to provide tools this agent can use
    fn get_tools(&self) -> Vec<Tool> {
        vec![]
    }

    // Optional: Override to execute tools by name
    fn execute_tool(&self, name: &str, _args: &serde_json::Value) -> Result<String> {
        Err(anyhow!("Unknown tool: {}", name))
    }

    // Core: Make a single LLM request
    async fn make_request(&self, messages: &Vec<Message>, tools: Option<Vec<Tool>>) -> Result<Message> {
        let request = ChatRequest {
            model: self.model().to_string(),
            messages: messages.clone(),
            stream: false,
            tools,
        };

        let response = self
            .client()
            .post(self.ollama_url())
            .json(&request)
            .send()
            .await?
            .json::<ChatResponse>()
            .await?;

        Ok(response.message)
    }

    // Agentic loop: Process an instruction using this agent's tools
    async fn run(&self, instruction: &str) -> Result<String> {
        let tools = self.get_tools();
        let tools_option = if tools.is_empty() { None } else { Some(tools.clone()) };

        let mut messages = vec![
            Message {
                role: "system".to_string(),
                content: Some(self.system_prompt().to_string()),
                tool_calls: None,
            },
            Message {
                role: "user".to_string(),
                content: Some(instruction.to_string()),
                tool_calls: None,
            },
        ];

        loop {
            let response = self.make_request(&messages, tools_option.clone()).await?;
            messages.push(response.clone());

            if let Some(tool_calls) = &response.tool_calls {
                for tool_call in tool_calls {
                    let arguments = &tool_call.function.arguments;
                    let result = self.execute_tool(
                        &tool_call.function.name,
                        &tool_call.function.arguments,
                    )?;

                    messages.push(Message {
                        role: "tool".to_string(),
                        content: Some(result),
                        tool_calls: None,
                    });
                }
            } else {
                // No tool calls - return final response
                return Ok(response.content.unwrap_or_default());
            }
        }
    }
}
