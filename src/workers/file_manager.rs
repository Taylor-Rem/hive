use std::path::PathBuf;
use std::fs;
use std::io::Result;
use dotenvy;
use crate::traits::{Worker, Capabilities, WorkerFactory};

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
