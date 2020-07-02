use crate::message::{Message};
use crate::events::{EventQueue, Event};

use std::time::{Duration};
use std::hash::{Hash};

pub enum Signal {
    NewArena,
    ArenaCreated,
    ComputeFrame,
}

pub struct ServerManager<Endpoint> {
    event_queue: EventQueue<Message, Signal, Endpoint>
}

impl<Endpoint: Hash + Copy> ServerManager<Endpoint> {
    pub fn new(event_queue: EventQueue<Message, Signal, Endpoint>) -> ServerManager<Endpoint> {
        ServerManager { event_queue }
    }

    pub fn run(&mut self) {
        loop {
            if let Some(event) = self.event_queue.pop_event(Duration::from_millis(3000)) {
                match event {
                    Event::Message(message, endpoint) => match message {
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

