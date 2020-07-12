use crate::message::{ClientMessage, ServerMessage};

use message_io::events::{EventQueue};
use message_io::network::{NetworkManager, NetEvent, TransportProtocol, Endpoint};

use std::net::{SocketAddr};


pub enum Event {
    Network(NetEvent<ClientMessage>),
}

pub struct ServerManager {
    event_queue: EventQueue<Event>,
    network: NetworkManager,
    //room here
}

impl ServerManager {
    pub fn new(addr: SocketAddr) -> Option<ServerManager> {
        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();
        let mut network = NetworkManager::new(move |net_event| network_sender.send(Event::Network(net_event)));

        network.listen(addr, TransportProtocol::Tcp).map(|_| {
            ServerManager {
                event_queue,
                network,
            }
        })
    }

    pub fn run(&mut self) {
        loop {
            match self.event_queue.receive() {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(message, endpoint) => {
                        //trace!(message, endpoint)
                        match message {
                            ClientMessage::Version{tag} => {
                                self.network.send(endpoint, ServerMessage::Version{tag: String::from("0.1.0"), compatible: true});
                            }
                        }
                    },
                    NetEvent::AddedEndpoint(_, _) => {},
                    NetEvent::RemovedEndpoint(_) => {}
                }
            }
        }
    }
}
