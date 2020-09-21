pub use crate::message::{LoginStatus};
use crate::message::{ClientMessage, ServerMessage, ServerInfo, LoggedKind};
use crate::version::{self, Compatibility};
use crate::util::{self};

use message_io::events::{Senderable};
use message_io::network::{NetworkManager, NetEvent, Endpoint};

use std::net::{IpAddr, SocketAddr};
use std::time::{Duration};


const UDP_HANDSHAKE_MAX_ATTEMPS: usize = 10;

#[derive(Debug)]
pub enum ConnectionResult {
    Connected,
    NotFound,
}

#[derive(Debug)]
pub enum InternalEvent {
    Network(NetEvent<ServerMessage>),
    HelloUdp(usize), //attempt
}

#[derive(Debug)]
pub enum ServerEvent {
    Internal(InternalEvent),
    Disconnected,
    CheckedVersion(String, Compatibility),
    ServerInfo(ServerInfo),
    PlayerListUpdated(Vec<String>),
    LoginStatus(String, LoginStatus),
    UdpReachable,
    StartGame,
    FinishGame,
    PrepareArena(Duration),
    StartArena,
    FinishArena,
    ArenaStep,
}

struct ConnectionInfo {
    ip: IpAddr,
    udp_port: Option<u16>,
    tcp: Option<Endpoint>,
    udp: Option<Endpoint>,
    has_udp_hasdshake: bool,
    session_token: Option<usize>,
}

pub struct ServerConnection {
    network: NetworkManager,
    connection: ConnectionInfo,
    sender: Box<dyn Senderable<ServerEvent>>
}

impl ServerConnection {
    pub fn new<S>(addr: SocketAddr, sender: S) -> ServerConnection
    where S: Senderable<ServerEvent> + Send + 'static + Clone {
        let network_sender = sender.clone();
        let network = NetworkManager::new(move |net_event| {
            network_sender.send(ServerEvent::Internal(InternalEvent::Network(net_event)))
        });

        ServerConnection {
            network,
            connection: ConnectionInfo {
                ip: addr.ip(),
                udp_port: None,
                tcp: None,
                udp: None,
                has_udp_hasdshake: false,
                session_token: None,
            },
            sender: Box::new(sender),
        }
    }

    pub fn connect(&mut self, addr: SocketAddr) -> ConnectionResult {
        match self.network.connect_tcp(addr) {
            Ok(tcp_endpoint) => {
                log::info!("Connected to server by tcp on {}", addr);
                self.connection.tcp = Some(tcp_endpoint);
                ConnectionResult::Connected
            },
            Err(_) => {
                log::error!("Could not connect to server by tcp on {}", addr);
                ConnectionResult::NotFound
            }
        }
    }

    pub fn check_version(&mut self) {
        let tcp = *self.connection.tcp.as_ref().unwrap();
        self.network.send(tcp, ClientMessage::Version(version::current().into())).unwrap();
    }

    pub fn subscribe_info(&mut self) {
        let tcp = *self.connection.tcp.as_ref().unwrap();
        self.network.send(tcp, ClientMessage::SubscribeServerInfo).unwrap();
    }

    pub fn login(&mut self, name: String) {
        let tcp = *self.connection.tcp.as_ref().unwrap();
        self.network.send(tcp, ClientMessage::Login(name)).unwrap();
    }

    pub fn logout(&mut self) {
        let tcp = *self.connection.tcp.as_ref().unwrap();
        self.network.send(tcp, ClientMessage::Logout).unwrap();
    }

    pub fn move_player(&mut self) {
        todo!();
    }

    pub fn cast_skill(&mut self) {
        todo!();
    }

    pub fn process_internal_event(&mut self, event: InternalEvent) {
        match event {
            InternalEvent::HelloUdp(attempt) => {
                self.process_hello_udp(attempt);
            }
            InternalEvent::Network(net_event) => match net_event {
                NetEvent::AddedEndpoint(_) => unreachable!(),
                NetEvent::RemovedEndpoint(_) => {
                    self.sender.send_with_priority(ServerEvent::Disconnected)
                },
                NetEvent::Message(endpoint, message) => {
                    log::trace!("Message from {}", endpoint.addr());
                    match message {
                        ServerMessage::Version(server_version, server_side_compatibility) => {
                            self.process_version(server_version, server_side_compatibility);
                        },
                        ServerMessage::ServerInfo(info) => {
                            self.process_server_info(info);
                        },
                        ServerMessage::DynamicServerInfo(players) => {
                            self.process_dynamic_server_info(players);
                        },
                        ServerMessage::LoginStatus(player, status) => {
                            self.process_login_status(player, status);
                        },
                        ServerMessage::UdpConnected => {
                            self.process_udp_connected();
                        },
                        ServerMessage::StartGame => {
                            self.process_start_game();
                        },
                        ServerMessage::FinishGame => {
                            self.process_finish_game();
                        },
                        ServerMessage::PrepareArena(duration) => {
                            self.process_prepare_arena(duration);
                        },
                        ServerMessage::StartArena => {
                            self.process_start_arena();
                        },
                        ServerMessage::FinishArena => {
                            self.process_finish_arena();
                        },
                        ServerMessage::Step => {
                            self.process_arena_step();
                        },
                    }
                },
            },
        }
    }

    fn process_version(&mut self, server_version: String, server_side_compatibility: Compatibility) {
        let client_side_compatibility = version::check(version::current(), &server_version);
        let compatibility = std::cmp::min(client_side_compatibility, server_side_compatibility);
        match compatibility {
            Compatibility::Fully => {
                log::info!("Fully compatible versions {}", version::current());
            }
            Compatibility::OkOutdated => {
                log::warn!("Compatible server version but differs. Client: {}. Server: {}", version::current(), server_version);
            }
            Compatibility::None => {
                log::error!("Incompatible server version. Client: {}. Server: {}", version::current(), server_version);
            }
        }

        self.sender.send(ServerEvent::CheckedVersion(server_version, server_side_compatibility));
    }

    fn process_server_info(&mut self, info: ServerInfo) {
        log::info!("Server info: {:?}", info);
        self.connection.udp_port = Some(info.udp_port);
        self.sender.send(ServerEvent::ServerInfo(info));
    }

    fn process_dynamic_server_info(&mut self, player_names: Vec<String>) {
        log::info!("Player list updated: {}", util::format::player_names(&player_names));
        self.sender.send(ServerEvent::PlayerListUpdated(player_names));
    }

    fn process_login_status(&mut self, player_name: String, status: LoginStatus) {
        match status {
            LoginStatus::Logged(token, kind) => {
                let kind_str = match kind {
                    LoggedKind::FirstTime => "Logged",
                    LoggedKind::Reconnection => "Reconnected",
                };
                log::info!("{} with name '{}' successful. Token Id: {}", kind_str, player_name, token);

                let udp_port = *self.connection.udp_port.as_ref().unwrap();
                self.connection.session_token = Some(token);
                self.connection.udp = Some(self.network.connect_udp((self.connection.ip, udp_port)).unwrap());
                log::info!("Connection by udp on port {}", udp_port);
                self.sender.send(ServerEvent::Internal(InternalEvent::HelloUdp(0)));
            },
            LoginStatus::InvalidPlayerName => {
                log::warn!("Invalid character name {}", player_name);
            },
            LoginStatus::AlreadyLogged => {
                log::warn!("Character name '{}' already logged", player_name);
            },
            LoginStatus::PlayerLimit => {
                log::error!("Server full");
            },
        }
        self.sender.send(ServerEvent::LoginStatus(player_name, status));
    }

    fn process_hello_udp(&mut self, attempt: usize) {
        if !self.connection.has_udp_hasdshake {
            match self.connection.session_token {
                Some(token) => match self.connection.udp {
                    Some(udp_endpoint) =>
                        if attempt < UDP_HANDSHAKE_MAX_ATTEMPS {
                            log::trace!("Udp handshake attempt: {}", attempt);
                            self.network.send(udp_endpoint, ClientMessage::ConnectUdp(token)).unwrap();
                            let next_message_timer = Duration::from_millis((attempt * attempt) as u64 + 1);
                            self.sender.send_with_timer(ServerEvent::Internal(InternalEvent::HelloUdp(attempt + 1)), next_message_timer);
                        }
                        else {
                            log::warn!("Unable to communicate by udp.");
                        }
                    None => log::warn!("Attempt to send hello udp without known endpoint"),
                },
                None => log::warn!("Attempt to send hello udp without logged session"),
            }
        }
    }

    fn process_udp_connected(&mut self) {
        let tcp = *self.connection.tcp.as_ref().unwrap();
        self.network.send(tcp, ClientMessage::TrustUdp).unwrap();
        self.connection.has_udp_hasdshake = true;
        log::info!("Udp successful connected");
        self.sender.send(ServerEvent::UdpReachable);
    }

    fn process_start_game(&mut self) {
        log::info!("Start game");
        self.sender.send(ServerEvent::StartGame);
    }

    fn process_finish_game(&mut self) {
        log::info!("Finish game");
        self.connection.has_udp_hasdshake = false;
        self.sender.send(ServerEvent::FinishGame);
    }

    fn process_prepare_arena(&mut self, duration: Duration) {
        log::info!("The arena will be start in {}", duration.as_secs_f32());
        self.sender.send(ServerEvent::PrepareArena(duration));
    }

    fn process_start_arena(&mut self) {
        log::info!("Start arena");
        self.sender.send(ServerEvent::StartArena);
    }

    fn process_finish_arena(&mut self) {
        log::info!("Finish arena");
        self.sender.send(ServerEvent::FinishArena);
    }

    fn process_arena_step(&mut self) {
        log::info!("Process arena step");
        self.sender.send(ServerEvent::ArenaStep);
    }
}

