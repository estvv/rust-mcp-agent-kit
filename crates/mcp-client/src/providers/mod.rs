// src/providers/mod.rs

mod ollama;

pub mod __mock__ {
    pub mod mock;
}

pub use ollama::OllamaProvider;
pub use __mock__::mock::MockProvider;
