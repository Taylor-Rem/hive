use reqwest::Client;

pub trait Agent {
    fn ollama_url(&self) -> &'static str;
    fn model(&self) -> &'static str;
    fn system_prompt(&self) -> &'static str;
    fn client(& self) -> Client;
    
}