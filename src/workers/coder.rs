use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use crate::traits::{Worker, WorkerFactory, Agent};

inventory::submit! {
    WorkerFactory(|| Box::new(Coder::new()))
}

pub struct Coder;

#[async_trait]
impl Worker for Coder {
    fn role(&self) -> &'static str {
        "coder"
    }

    fn description(&self) -> &'static str {
        "Analyzes code, writes code, and provides technical solutions. Give full context and code."
    }

    fn worker_type(&self) -> &'static str {
        "advanced"
    }

    async fn process(&self, instruction: &str) -> Result<String> {
        Agent::run(self, instruction).await
    }
}

impl Agent for Coder {
    fn ollama_url(&self) -> &'static str { "http://localhost:11435/api/chat" /* P40 (GPU 1) */ }
    fn model(&self) -> &'static str { "qwen2.5-coder:32b" }
    fn client(&self) -> Client { Client::new() }
    fn _type(&self) -> &'static str { "advanced" }
    fn system_prompt(&self) -> &'static str { SYSTEM_PROMPT }
}

impl Coder {
    pub fn new() -> Self {
        Coder
    }
}

const SYSTEM_PROMPT: &str = r#"You are an expert software engineer. You analyze code and write code.

# When Analyzing Code
- Identify issues, bugs, or improvements
- Explain the code's purpose and structure
- Point out potential problems or edge cases

# When Writing Code
- Write clean, idiomatic code
- Follow best practices for the language
- Include necessary error handling
- Keep it focused on the specific request

# Response Format
- Be direct and technical
- When writing code, output the code directly
- When analyzing, be concise but thorough
- No unnecessary preamble or filler"#;
