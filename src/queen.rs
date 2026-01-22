use std::collections::HashMap;
use reqwest::Client;
use crate::traits::{Agent, Worker, WorkerFactory};

pub struct Queen {
    workers: HashMap<&'static str, Box<dyn Worker>>
}
impl Agent for Queen {
    fn ollama_url(&self) -> &'static str {
        "http://localhost:8000/api/chat"
    }
    fn model(&self) -> &'static str {
        ""
    }
    fn system_prompt(&self) -> &'static str {
        "you are a helpful ai assistant"
    }
    fn client(&self) -> Client {
        Client::new()
    }
}

impl Queen {
    pub fn new() -> Queen {
        let workers = inventory::iter::<WorkerFactory>
            .into_iter()
            .map(|factory| {
                let worker = (factory.0)();
                (worker.role(), worker)
            })
            .collect();

        Queen { workers }
    }
}