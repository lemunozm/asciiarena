use crate::message::{ClientMessage, ServerMessage};
use crate::version::{self, Compatibility};

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
        self.network.send(self.server, ClientMessage::Version(version::current().to_string()));
        loop {
            match self.event_queue.receive() {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(message, endpoint) => {
                        log::trace!("Message from {}: {:?}", self.network.endpoint_remote_address(endpoint).unwrap(), message);
                        match message {
                            ServerMessage::Version(server_version, compatibility) => {
                                match compatibility {
                                    Compatibility::Fully =>
                                        log::trace!("Fully compatible versions {}", version::current()),
                                    Compatibility::OkOutdated => {
                                        log::warn!("Compatible server version but differs. Client: {}. Server: {}", version::current(), server_version);
                                        println!("Compatible versions but it is recomendable to update. Client: {}. Server: {}", version::current(), server_version);
                                    }
                                    Compatibility::None => {
                                        log::error!("Incompatible server version. Client: {}. Server: {}", version::current(), server_version);
                                        println!("Incompatible server version. Client: {}. Server: {}", version::current(), server_version);
                                        return None; //Specify errors
                                    }
                                }
                            }
                        }
                    },
                    NetEvent::AddedEndpoint(_, _) => unreachable!(),
                    NetEvent::RemovedEndpoint(_) => {
                        println!("Closing client");
                        return None;
                    }
                }
            }
        }
    }
}


