use phf::Map;
pub type Capabilities = Map<&'static str, &'static str>;

pub trait Worker {
    fn role(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn capabilities(&self) -> &'static Capabilities;
}

/// Factory function type for creating workers
pub struct WorkerFactory(pub fn() -> Box<dyn Worker>);

inventory::collect!(WorkerFactory);