use crate::message::{ClientMessage, ServerMessage};
use crate::version::{self, Compatibility};

use message_io::events::{EventQueue};
use message_io::network::{NetworkManager, NetEvent, TransportProtocol, Endpoint};

use std::net::{SocketAddr};

#[derive(Debug)]
pub enum ClosingReason {
    //Finished,
    Forced,
    ConnectionLost,
    IncompatibleVersions,
}

#[derive(Debug)]
enum Event {
    Network(NetEvent<ServerMessage>),
    Close(ClosingReason),
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

        let network_sender = event_queue.sender().clone();
        ctrlc::set_handler(move || network_sender.send_with_priority(Event::Close(ClosingReason::Forced))).unwrap();

        network.connect(addr, TransportProtocol::Tcp).map(|(server, _)| {
            log::info!("Connected to server on '{}' by tcp", addr);
            ClientManager {
                event_queue,
                network,
                server,
            }
        })
    }

    pub fn run(&mut self) -> ClosingReason {
        self.network.send(self.server, ClientMessage::Version(version::current().to_string()));
        loop {
            let event = self.event_queue.receive();
            log::trace!("[Process event] - {:?}", event);
            match event {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(message, endpoint) => {
                        log::trace!("Message from {}", self.network.endpoint_remote_address(endpoint).unwrap());
                        match message {
                            ServerMessage::Version(server_version, server_side_compatibility) =>
                                self.process_version(&server_version, server_side_compatibility),
                        }
                    },
                    NetEvent::AddedEndpoint(_, _) => unreachable!(),
                    NetEvent::RemovedEndpoint(_) => {
                        println!("Connection lost with the server");
                        self.event_queue.sender().send_with_priority(Event::Close(ClosingReason::ConnectionLost))
                    }
                },
                Event::Close(reason) => {
                    log::info!("Closing client. Reason: {:?}", reason);
                    break reason
                }
            }
        }
    }

    fn process_version(&mut self, server_version: &str, server_side_compatibility: Compatibility) {
        let client_side_compatibility = version::check(version::current(), &server_version);
        let compatibility = std::cmp::min(client_side_compatibility, server_side_compatibility);
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
                self.event_queue.sender().send_with_priority(Event::Close(ClosingReason::IncompatibleVersions));
            }
        }
    }
}


