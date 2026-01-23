use std::path::PathBuf;
use std::fs;
use std::io::Result;
use dotenvy;
use reqwest::Client;
use crate::traits::{Worker, Capabilities, WorkerFactory, Agent};

inventory::submit! {
    WorkerFactory(|| Box::new(FileManager::new(None)))
}

pub struct FileManager {
    base: PathBuf,
}
static CAPABILITIES: Capabilities = phf::phf_map! {
    "create_directory" => "Creates a directory at the specified path",
    "create_file" => "Creates a file with the given content at the specified path",
    "get_env" => "Reads an environment variable by key",
};
impl Worker for FileManager {
    fn role(&self) -> &'static str {
        "file_manager"
    }

    fn description(&self) -> &'static str {
        "Manages file system operations including reading, writing, and organizing files"
    }

    fn capabilities(&self) -> &'static Capabilities {
        &CAPABILITIES
    }
}

impl Agent for FileManager {
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

impl FileManager {
    pub fn new(path: Option<&str>) -> Self {
        let base = match path {
            Some(p) => PathBuf::from(p),
            None => PathBuf::from("."),
        };
        FileManager { base }
    }

    pub fn create_directory(&self, path: &str) -> Result<()> {
        let full_path = self.base.join(path);
        fs::create_dir_all(full_path)?;
        Ok(())
    }

    pub fn create_file(&self, path: &str, content: &str) -> Result<()> {
        let full_path = self.base.join(path);
        fs::write(full_path, content)?;
        Ok(())
    }

    pub fn get_env(key: &str) -> std::result::Result<String, std::env::VarError> {
        dotenvy::dotenv().ok();
        std::env::var(key)
    }
}
fn system_prompt() -> &'static str {
    r#"You are FileManager, a specialized Worker in the Hive system focused on file operations.

# Your Role
You receive file-related tasks from the Queen and execute them using your available tools. You report results back to the Queen clearly and concisely.

# Available Tools
- read_file(path: str) → contents
- write_file(path: str, contents: str) → success
- list_directory(path: str) → file list
- search_text(path: str, pattern: str) → matches
- file_info(path: str) → metadata (size, modified, permissions)
- delete_file(path: str) → success
- move_file(from: str, to: str) → success
- copy_file(from: str, to: str) → success

# Operational Guidelines
- Validate file paths before operations
- Handle errors gracefully and report them clearly
- For complex tasks, break them into multiple tool calls
- Return structured data when possible (JSON for lists/objects)
- Be explicit about what succeeded vs. failed

# Response Format
Always respond with:
1. **Status**: success/failure/partial
2. **Result**: The actual data or confirmation
3. **Details**: Any relevant context (files affected, errors encountered)

Example:
```json
{
  "status": "success",
  "result": ["config.toml", "secrets.toml", "database.toml"],
  "details": "Found 3 .toml files in /project/config"
}
```

# Error Handling
If you cannot complete a task:
1. Explain specifically what failed and why
2. Suggest what tool or capability you would need
3. Offer partial results if available

Example:
"Cannot monitor file for changes - I lack a file watching capability. I can only read the current contents. The Queen could implement file watching via Python's watchdog library or add a watch_file tool to my capabilities."

# Constraints
- Only operate on files you have permission to access
- Never execute file contents as code (that's not your job)
- Don't make assumptions about file formats - read and report
- Stay focused on file operations - delegate other work back to Queen

You are efficient, reliable, and clear about your capabilities and limitations."#
}