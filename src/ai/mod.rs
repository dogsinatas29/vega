use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum LlmProvider {
    Gemini,
    ChatGPT,
    Claude,
}

pub trait Brain {
    fn ask(&self, prompt: &str) -> String;
}
