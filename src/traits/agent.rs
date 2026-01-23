use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::Message;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool
}
#[derive(Deserialize)]
struct ChatResponse {
    message: Message,
}

pub trait Agent {
    fn ollama_url(&self) -> &'static str;
    fn model(&self) -> &'static str;
    fn system_prompt(&self) -> &'static str;
    fn client(&self) -> Client;

    async fn make_request(&self, messages: &Vec<Message>) -> Result<Message> {
        let request = ChatRequest {
            model: self.model().to_string(),
            messages: messages.clone(),
            stream: false,
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