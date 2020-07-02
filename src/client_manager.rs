use crate::message::{Message};
use crate::events::{EventQueue, Event};

use std::time::{Duration};
use std::hash::{Hash};

pub struct ClientManager<Endpoint> {
    event_queue: EventQueue<Message, (), Endpoint>,
    server: Endpoint,
}

impl<Endpoint: Hash + Copy> ClientManager<Endpoint> {
    pub fn new(event_queue: EventQueue<Message, (), Endpoint>, server: Endpoint) -> ClientManager<Endpoint> {
        ClientManager { event_queue, server }
    }

    pub fn run(&mut self) {
        let message = Message::Version{ value: String::from("0.1.0") };
        self.event_queue.emit_message(message, self.server);
        loop {
            if let Some(event) = self.event_queue.pop_event(Duration::from_millis(50)) {
                match event {
                    Event::Message(message, endpoint) => match message {
                        Message::VersionInfo { value, compatible } => {
                            println!("version received");
                        },
                        _ => unreachable!()
                    },
                    _ => unreachable!()
                }
            }
        }
    }
}


