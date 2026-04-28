// src/providers/mod.rs

mod ollama;
mod openai;
mod mock;

pub use ollama::OllamaProvider;
pub use openai::OpenAIProvider;
pub use mock::MockProvider;
