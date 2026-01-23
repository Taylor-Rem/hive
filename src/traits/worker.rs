use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Worker: Send + Sync {
    /// Unique identifier for this worker
    fn role(&self) -> &'static str;

    /// Human-readable description of what this worker does
    fn description(&self) -> &'static str;

    /// Process an instruction and return the result
    /// Workers implement this using their own Agent capabilities
    async fn process(&self, instruction: &str) -> Result<String>;
}

/// Factory function type for creating workers
pub struct WorkerFactory(pub fn() -> Box<dyn Worker + Send + Sync>);

inventory::collect!(WorkerFactory);
