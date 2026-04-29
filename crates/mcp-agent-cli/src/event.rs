// src/event.rs

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
    LlmStreamContent(String),
    LlmStreamReasoning(String),
    LlmStreamToolCalls { calls: Vec<(String, String)> },
    LlmStreamDone,
    LlmError(String),
}

pub struct EventLoop {
    receiver: UnboundedReceiver<Event>,
    sender: UnboundedSender<Event>,
}

impl EventLoop {
    pub fn new(tick_rate: Duration) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let event_sender = sender.clone();
        let tick_sender = sender.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tick_rate);
            loop {
                interval.tick().await;
                let _ = tick_sender.send(Event::Tick);
            }
        });

        tokio::spawn(async move {
            loop {
                if let Ok(true) = event::poll(Duration::from_millis(50)) {
                    if let Ok(crossterm_event) = event::read() {
                        match crossterm_event {
                            CrosstermEvent::Key(key) => {
                                let _ = event_sender.send(Event::Key(key));
                            }
                            CrosstermEvent::Mouse(mouse) => {
                                let _ = event_sender.send(Event::Mouse(mouse));
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Self { receiver, sender }
    }

    pub fn sender(&self) -> UnboundedSender<Event> {
        self.sender.clone()
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.receiver.recv().await
    }
}
