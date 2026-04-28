// src/dispatcher.rs

use crate::command::{Command, CommandEntry};
use crate::types::RpcRequest;
use std::collections::HashMap;

pub struct Dispatcher {
    handlers: HashMap<&'static str, &'static (dyn Command + Send + Sync)>,
}

impl Dispatcher {
    pub fn new() -> Self {
        let mut handlers = HashMap::new();

        for entry in inventory::iter::<CommandEntry> {
            handlers.insert(entry.method, entry.command);
        }

        Dispatcher { handlers }
    }

    pub fn dispatch(&self, request: &RpcRequest) -> Option<serde_json::Value> {
        self.handlers.get(request.method.as_str()).map(|handler| handler.execute(request.clone()))
    }
}
