use crate::message::{Message};
use crate::events::{EventQueue, Event};

use std::time::{Duration};

pub enum Signal {
    NewArena,
    ArenaCreated,
    ComputeFrame,
}

pub struct ServerManager {
    event_queue: EventQueue<Message, Signal>
}

impl ServerManager {
    pub fn new(event_queue: EventQueue<Message, Signal>) -> ServerManager {
        ServerManager { event_queue }
    }

    pub fn run(&mut self) {
        loop {
            if let Some((event, endpoint)) = self.event_queue.pop_event(Duration::from_millis(50)) {
                match event {
                    Event::Message(message) => match message {
                        Message::Version { value } => {
                            let message = Message::VersionInfo { value: String::from("0.1.0"), compatible: true };
                            self.event_queue.emit_message(message, endpoint);
                            println!("version received");
                        },
                        _ => unreachable!()
                    },
                    Event::Signal(signal) => match signal {
                        _ => todo!()
                    }
                    _ => unreachable!()
                }
            }
        }
    }
}

