// src/lib.rs

pub mod client;
pub mod profile;
pub mod providers;
pub mod server_process;
pub mod orchestrator;

pub use client::{ChatClient, ChatResponse, Message, ToolCall, ToolDefinition};
pub use profile::Profile;
pub use server_process::ServerProcess;
pub use providers::OllamaProvider;
pub use providers::__mock__::mock::MockProvider;
pub use orchestrator::Orchestrator;

