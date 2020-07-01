use crate::message::{Message};
use crate::events::{EventQueue, Event, Endpoint};

use std::time::{Duration};

pub struct ClientManager {
    event_queue: EventQueue<Message, ()>,
    server: Endpoint,
}

impl ClientManager {
    pub fn new(event_queue: EventQueue<Message, ()>, server: Endpoint) -> ClientManager {
        ClientManager { event_queue, server }
    }

    pub fn run(&mut self) {
        let message = Message::Version{ value: String::from("0.1.0") };
        self.event_queue.emit_message(message, self.server);
        loop {
            if let Some((event, _)) = self.event_queue.pop_event(Duration::from_millis(50)) {
                match event {
                    Event::Message(message) => match message {
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


