use std::io::{self, Write};
use anyhow::Result;
use serde::{Deserialize, Serialize};

mod traits;
mod queen;
mod workers;

use queen::*;
use traits::Agent;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ToolCall {
    pub function: FunctionCall,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: serde_json::Value,
}
#[tokio::main]
async fn main() -> Result<()> {
    let queen = Queen::new();
    let mut messages = vec![Message {
        role: "system".to_string(),
        content: Some(queen.system_prompt().to_string()),
        tool_calls: None,
    }];
    println!("Queen is ready. Type 'quit' to exit.\n");

    loop {
        let input = wait_for_user_input()?;

        if input.eq_ignore_ascii_case("quit") {
            println!("Goodbye!");
            break;
        }

        if input.is_empty() {
            continue;
        }

        // Add user message
        messages.push(Message {
            role: "user".to_string(),
            content: Some(input),
            tool_calls: None,
        });

        // Agentic loop: keep processing until we get a final response
        let final_response = queen.run_agentic_loop(&mut messages).await?;

        println!("\nQueen: {}\n", final_response);
    }

    Ok(())
}

fn wait_for_user_input() -> Result<String> {
    print!("You: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}