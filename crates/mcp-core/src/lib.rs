// src/lib.rs

mod commands;
pub mod constants;
pub mod dispatcher;
pub mod server;
pub mod tool;
pub mod types;

mod command;
pub use command::{Command, CommandEntry};
pub use tool::{Tool, ToolEntry, ToolError};
pub use server::Server;
pub use dispatcher::Dispatcher;