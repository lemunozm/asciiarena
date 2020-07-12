use crate::message::{ClientMessage, ServerMessage};

use message_io::events::{EventQueue};
use message_io::network::{NetworkManager, NetEvent, TransportProtocol, Endpoint};

use std::net::{SocketAddr};


pub enum Event {
    Network(NetEvent<ServerMessage>),
}

pub struct ClientManager {
    event_queue: EventQueue<Event>,
    network: NetworkManager,
    server: Endpoint,
}

impl ClientManager {
    pub fn new(addr: SocketAddr) -> Option<ClientManager> {
        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();
        let mut network = NetworkManager::new(move |net_event| network_sender.send(Event::Network(net_event)));

        network.connect(addr, TransportProtocol::Tcp).map(|(server, _)| {
            ClientManager {
                event_queue,
                network,
                server,
            }
        })
    }

    pub fn run(&mut self) -> Option<()> {
        self.network.send(self.server, ClientMessage::Version{tag: String::from("0.1.0")});
        loop {
            match self.event_queue.receive() {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(message, endpoint) => {
                        //trace!(message, endpoint)
                        match message {
                            ServerMessage::Version{tag, compatible} => {
                            }
                        }
                    },
                    NetEvent::AddedEndpoint(_, _) => unreachable!(),
                    NetEvent::RemovedEndpoint(_) => {
                        return None;
                    }
                }
            }
        }
    }
}


