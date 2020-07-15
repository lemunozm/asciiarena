use crate::message::{ClientMessage, ServerMessage, ServerInfo};
use crate::version::{self, Compatibility};

use message_io::events::{EventQueue};
use message_io::network::{NetworkManager, NetEvent, TransportProtocol, Endpoint};

use std::net::{SocketAddr};

#[derive(Debug)]
enum Event {
    Network(NetEvent<ClientMessage>),
    Close,
}

pub struct ServerConfig {
    pub tcp_port: u16,
    pub udp_port: u16,
    pub players: usize,
    pub map_dimension: (usize, usize),
    pub winner_points: usize,
}

pub struct ServerManager {
    event_queue: EventQueue<Event>,
    network: NetworkManager,
    config: ServerConfig,
    //room here
}

impl ServerManager {
    pub fn new(config: ServerConfig) -> Option<ServerManager> {
        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();
        let mut network = NetworkManager::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let network_sender = event_queue.sender().clone();
        ctrlc::set_handler(move || network_sender.send_with_priority(Event::Close)).unwrap();

        if let None = network.listen(SocketAddr::from(([0, 0, 0, 0], config.tcp_port)), TransportProtocol::Tcp) {
            log::error!("Can not run server on tcp port {}", config.tcp_port);
            return None;
        }

        if let None = network.listen(SocketAddr::from(([0, 0, 0, 0], config.udp_port)), TransportProtocol::Udp) {
            log::error!("Can not run server on udp port {}", config.udp_port);
            return None;
        }

        log::info!("Server running on tcp ports {} (tcp) and {} (udp)", config.tcp_port, config.udp_port);
        Some(ServerManager {
            event_queue,
            network,
            config,
        })
    }

    pub fn run(&mut self) {
        loop {
            let event = self.event_queue.receive();
            log::trace!("[Process event] - {:?}", event);
            match event {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(message, endpoint) => {
                        log::trace!("Message from {}", self.network.endpoint_remote_address(endpoint).unwrap());
                        match message {
                            ClientMessage::Version(client_version) =>
                                self.process_version(endpoint, &client_version),
                            ClientMessage::RequestServerInfo =>
                                self.process_request_server_info(endpoint),
                        }
                    },
                    NetEvent::AddedEndpoint(_, _) => (),
                    NetEvent::RemovedEndpoint(_) => {},
                },
                Event::Close => {
                    log::info!("Closing server");
                    break
                }
            }
        }
    }

    fn process_version(&mut self, endpoint: Endpoint, client_version: &str) {
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

    fn process_request_server_info(&mut self, endpoint: Endpoint) {
        let info = ServerInfo {
            udp_port: self.config.udp_port,
            players: self.config.players as u8,
            map_dimension: (self.config.map_dimension.0 as u16, self.config.map_dimension.1 as u16),
            winner_points: self.config.winner_points as u16,
            logged_players: Vec::new(), //TODO
        };

        self.network.send(endpoint, ServerMessage::ServerInfo(info));
    }
}
