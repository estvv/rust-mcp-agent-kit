// src/lib.rs

pub mod client;
pub mod providers;
pub mod server_process;

pub use client::{ChatClient, ChatResponse, Message, ToolCall, ToolDefinition};
pub use server_process::ServerProcess;
pub use providers::{MockProvider, OllamaProvider, OpenAIProvider};
