// src/server.rs

use crate::dispatcher::Dispatcher;
use crate::types::RpcRequest;
use std::io::{self, BufRead};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct Server {
    dispatcher: Dispatcher,
    running: Arc<AtomicBool>,
}

impl Server {
    pub fn new() -> Self {
        Server {
            dispatcher: Dispatcher::new(),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn run(&self) {
        let stdin = io::stdin();
        let mut lines = stdin.lock().lines();

        while self.running.load(Ordering::SeqCst) {
            let line = match lines.next() {
                Some(Ok(l)) => l,
                Some(Err(e)) => {
                    eprintln!("Read error: {}", e);
                    continue;
                }
                None => break,
            };

            let request: RpcRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    eprintln!("Parsing error: {}", e);
                    continue;
                }
            };

            if request.method == "shutdown" {
                if let Some(response) = self.dispatcher.dispatch(&request) {
                    println!("{}", response);
                }
                self.running.store(false, Ordering::SeqCst);
                break;
            }

            if request.id.is_some() {
                match self.dispatcher.dispatch(&request) {
                    Some(response) => println!("{}", response),
                    None => eprintln!("Unknown method: {}", request.method),
                }
            }
        }
    }
}
