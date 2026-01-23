use std::io::{self, Write};
use anyhow::Result;
use serde::{Deserialize, Serialize};

mod traits;
mod queen;
mod workers;

use queen::*;
use traits::Agent;
#[derive(Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}
#[tokio::main]
async fn main() -> Result<()> {
    let queen = Queen::new();
    let mut messages = vec![Message {
        role: "system".to_string(),
        content: queen.system_prompt().to_string()
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
            content: input,
        });

        // Get response from queen
        let response = queen.make_request(&messages).await?;

        // Add assistant response to history
        messages.push(response.clone());

        println!("\nQueen: {}\n", response.content);
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