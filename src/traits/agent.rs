use anyhow::Result;
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

pub trait Agent {
    fn ollama_url(&self) -> &'static str;
    fn model(&self) -> &'static str;
    fn system_prompt(&self) -> &'static str;
    fn client(&self) -> Client;

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
}