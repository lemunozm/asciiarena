use crate::message::{ClientMessage, ServerMessage};
use crate::version::{self, Compatibility};

use message_io::events::{EventQueue};
use message_io::network::{NetworkManager, NetEvent, TransportProtocol};

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
    pub fn new(tcp_port: u16, udp_port: u16) -> Option<ServerManager> {
        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();
        let mut network = NetworkManager::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let tcp_listener = network.listen(SocketAddr::from(([0, 0, 0, 0], tcp_port)), TransportProtocol::Tcp);
        let udp_listener = network.listen(SocketAddr::from(([0, 0, 0, 0], udp_port)), TransportProtocol::Udp);

        tcp_listener.and(udp_listener).map(|_| {
            log::info!("Server running on tcp ports {} (tcp) and {} (udp)", tcp_port, udp_port);
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
                        log::trace!("Message from {}: {:?}", self.network.endpoint_remote_address(endpoint).unwrap(), message);
                        match message {
                            ClientMessage::Version(client_version) => {
                                log::debug!("Version request");
                                let compatibility = version::check(&client_version, version::current());
                                match compatibility {
                                    Compatibility::Fully =>
                                        log::trace!("Fully compatible versions: {}", client_version),
                                    Compatibility::OkOutdated =>
                                        log::info!("Compatible client version but differs. Client: {}. Server: {}", client_version, version::current()),
                                    Compatibility::None =>
                                        log::warn!("Incompatible client version. Client: {}. Server: {}. Rejected", client_version, version::current()),
                                };
                                self.network.send(endpoint, ServerMessage::Version(version::current().to_string(), compatibility));
                                if let Compatibility::None = compatibility {
                                    self.network.remove_endpoint(endpoint);
                                }
                            }
                        }
                    },
                    NetEvent::AddedEndpoint(_, _) => (),
                    NetEvent::RemovedEndpoint(_) => {}
                }
            }
        }
    }
}
