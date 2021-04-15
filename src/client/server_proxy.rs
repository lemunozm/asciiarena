use crate::message::{
    LoginStatus, ServerInfo, ClientMessage, ServerMessage, LoggedKind, GameInfo, ArenaInfo, Frame,
    GameEvent,
};
use crate::encoding::{self, Encoder};
use crate::version::{self, Compatibility};
use crate::direction::{Direction};
use crate::ids::{SkillId};

use message_io::node::{self, NodeHandler, NodeTask, NodeEvent};
use message_io::network::{Endpoint, Transport, NetEvent};

use std::net::{IpAddr, SocketAddr};
use std::time::{Duration};

const UDP_HANDSHAKE_MAX_ATTEMPS: usize = 10;

lazy_static! {
    static ref EVENT_SAMPLING_TIMEOUT: Duration = Duration::from_millis(50);
}

/// Server API requests
#[derive(Debug)]
pub enum ApiCall {
    Connect(SocketAddr),
    Disconnect,
    CheckVersion(String),
    SubscribeInfo,
    Login(char),
    Logout,
    MovePlayer(Direction),
    CastSkill(Direction, SkillId),
}

/// API Events from server
#[derive(Debug)]
pub enum ServerEvent {
    ConnectionResult(ConnectionStatus),
    CheckedVersion(String, Compatibility),
    StaticServerInfo(ServerInfo),
    DynamicServerInfo(Vec<char>),
    LoginStatus(LoginStatus),
    UdpReachable(bool),
    StartGame(GameInfo),
    FinishGame,
    GameEvent(GameEvent),
    GameStep(Frame),
    WaitArena(Duration),
    StartArena(ArenaInfo),
}

#[derive(Debug, Clone, Copy)]
pub enum ConnectionStatus {
    Connected,
    NotConnected,
    NotFound,
    Lost,
}

impl ConnectionStatus {
    pub fn is_connected(&self) -> bool {
        match self {
            ConnectionStatus::Connected => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
enum ProxyEvent {
    Api(ApiCall),
    HelloUdp(usize),
}

pub struct ServerProxy {
    node: NodeHandler<ProxyEvent>,
    node_task: NodeTask,
}

impl ServerProxy {
    pub fn new(event_callback: impl Fn(ServerEvent) + Send + 'static) -> ServerProxy {
        let (node, listener) = node::split();

        let mut connection = ServerConnection::new(node.clone());
        let node_task = listener.for_each_async(move |event| {
            connection.process_event(event, |server_event| event_callback(server_event));
        });

        ServerProxy { node, node_task }
    }

    pub fn api(&mut self) -> ServerApi {
        return ServerApi { node: self.node.clone() }
    }
}

impl Drop for ServerProxy {
    fn drop(&mut self) {
        self.node.stop();
        self.node_task.wait();
    }
}

pub struct ServerApi {
    node: NodeHandler<ProxyEvent>,
}

impl ServerApi {
    pub fn call(&mut self, api_call: ApiCall) {
        self.node.signals().send(ProxyEvent::Api(api_call));
    }
}

struct ConnectionInfo {
    ip: Option<IpAddr>,
    udp_port: Option<u16>,
    tcp: Option<Endpoint>,
    udp: Option<Endpoint>,
    has_udp_hasdshake: bool,
    session_token: Option<usize>,
}

struct ServerConnection {
    node: NodeHandler<ProxyEvent>,
    encoder: Encoder,
    connection: ConnectionInfo,
}

impl ServerConnection {
    pub fn new(node: NodeHandler<ProxyEvent>) -> Self {
        Self {
            node,
            encoder: Encoder::new(),
            connection: ConnectionInfo {
                ip: None,
                udp_port: None,
                tcp: None,
                udp: None,
                has_udp_hasdshake: false,
                session_token: None,
            },
        }
    }

    fn send_to_server(&mut self, endpoint: Endpoint, message: ClientMessage) {
        self.node.network().send(endpoint, self.encoder.encode(message));
    }

    fn connect(&mut self, addr: SocketAddr) -> ConnectionStatus {
        self.disconnect(); // Ensure there is no connection, reset if there is.
        match self.node.network().connect(Transport::FramedTcp, addr) {
            Ok((tcp_endpoint, _)) => {
                log::info!("Connected to server by tcp on {}", addr);
                self.connection.tcp = Some(tcp_endpoint);
                self.connection.ip = Some(addr.ip());
                ConnectionStatus::Connected
            }
            Err(_) => {
                log::error!("Could not connect to server by tcp on {}", addr);
                ConnectionStatus::NotFound
            }
        }
    }

    fn disconnect(&mut self) -> ConnectionStatus {
        self.connection.has_udp_hasdshake = false;
        self.connection.session_token = None;
        self.connection.udp_port = None;
        self.connection.udp = None;
        self.connection.ip = None;
        if let Some(endpoint) = self.connection.tcp {
            self.node.network().remove(endpoint.resource_id());
            self.connection.tcp = None;
        }
        ConnectionStatus::NotConnected
    }

    pub fn logout(&mut self) {
        self.connection.has_udp_hasdshake = false;
        self.connection.session_token = None;
        self.connection.udp = None;
        let tcp = *self.connection.tcp.as_ref().unwrap();
        self.send_to_server(tcp, ClientMessage::Logout);
    }

    pub fn process_event(&mut self, event: NodeEvent<ProxyEvent>, callback: impl Fn(ServerEvent)) {
        log::trace!("[Process network event] - {:?}", event);
        match event {
            NodeEvent::Signal(signal) => match signal {
                ProxyEvent::Api(api_call) => match api_call {
                    ApiCall::Connect(addr) => {
                        let result = self.connect(addr);
                        callback(ServerEvent::ConnectionResult(result));
                    }
                    ApiCall::Disconnect => {
                        let result = self.disconnect();
                        callback(ServerEvent::ConnectionResult(result));
                    }
                    ApiCall::CheckVersion(version) => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.send_to_server(tcp, ClientMessage::Version(version));
                    }
                    ApiCall::SubscribeInfo => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.send_to_server(tcp, ClientMessage::SubscribeServerInfo);
                    }
                    ApiCall::Login(character) => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.send_to_server(tcp, ClientMessage::Login(character));
                    }
                    ApiCall::Logout => self.logout(),
                    ApiCall::MovePlayer(direction) => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.send_to_server(tcp, ClientMessage::MovePlayer(direction));
                    }
                    ApiCall::CastSkill(direction, id) => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.send_to_server(tcp, ClientMessage::CastSkill(direction, id));
                    }
                },
                ProxyEvent::HelloUdp(attempt) => self.process_hello_udp(attempt, callback),
            },
            NodeEvent::Network(net_event) => match net_event {
                NetEvent::Connected(_, _) => unreachable!(),
                NetEvent::Disconnected(_) => {
                    let result = ConnectionStatus::Lost;
                    callback(ServerEvent::ConnectionResult(result));
                }
                NetEvent::Message(endpoint, data) => match encoding::decode(&data) {
                    Some(message) => match message {
                        ServerMessage::Version(server_version, compatibility) => {
                            self.process_version(server_version, compatibility, callback);
                        }
                        ServerMessage::StaticServerInfo(info) => {
                            self.process_static_server_info(info, callback);
                        }
                        ServerMessage::DynamicServerInfo(players) => {
                            callback(ServerEvent::DynamicServerInfo(players));
                        }
                        ServerMessage::LoginStatus(character, status) => {
                            self.process_login_status(character, status, callback);
                        }
                        ServerMessage::UdpConnected => {
                            self.process_udp_connected(callback);
                        }
                        ServerMessage::StartGame(game_info) => {
                            callback(ServerEvent::StartGame(game_info));
                        }
                        ServerMessage::FinishGame => {
                            self.process_finish_game(callback);
                        }
                        ServerMessage::WaitArena(duration) => {
                            callback(ServerEvent::WaitArena(duration));
                        }
                        ServerMessage::StartArena(arena_info) => {
                            callback(ServerEvent::StartArena(arena_info));
                        }
                        ServerMessage::GameEvent(game_event) => {
                            callback(ServerEvent::GameEvent(game_event));
                        }
                        ServerMessage::GameStep(frame) => {
                            callback(ServerEvent::GameStep(frame));
                        }
                    },
                    None => {
                        log::error!(
                            "Server sends an unknown message. Connection rejected. \
                            Ensure the version compatibility.",
                        );
                        self.node.network().remove(endpoint.resource_id());
                    }
                },
            },
        }
    }

    fn process_version(
        &mut self,
        server_version: String,
        server_side_compatibility: Compatibility,
        callback: impl Fn(ServerEvent),
    ) {
        let client_side_compatibility = version::check(version::current(), &server_version);
        let compatibility = std::cmp::min(client_side_compatibility, server_side_compatibility);
        match compatibility {
            Compatibility::Fully => {
                log::info!("Fully compatible versions {}", version::current());
            }
            Compatibility::NotExact => {
                log::warn!(
                    "Compatible server version, but not exact. Client: {}. Server: {}",
                    version::current(),
                    server_version
                );
            }
            Compatibility::None => {
                log::error!(
                    "Incompatible server version. Client: {}. Server: {}",
                    version::current(),
                    server_version
                );
            }
        }

        callback(ServerEvent::CheckedVersion(server_version, compatibility));
    }

    fn process_static_server_info(&mut self, info: ServerInfo, callback: impl Fn(ServerEvent)) {
        self.connection.udp_port = Some(info.udp_port);
        callback(ServerEvent::StaticServerInfo(info));
    }

    fn process_login_status(
        &mut self,
        character: char,
        status: LoginStatus,
        callback: impl Fn(ServerEvent),
    ) {
        match status {
            LoginStatus::Logged(token, kind) => {
                let kind_str = match kind {
                    LoggedKind::FirstTime => "Logged",
                    LoggedKind::Reconnection => "Reconnected",
                };
                log::info!(
                    "{} with name '{}' successful. Token Id: {}",
                    kind_str,
                    character,
                    token
                );

                let udp_port = *self.connection.udp_port.as_ref().unwrap();
                let ip = *self.connection.ip.as_ref().unwrap();
                self.connection.session_token = Some(token);

                let addr = SocketAddr::new(ip, udp_port);
                let (endpoint, _) = self.node.network().connect(Transport::Udp, addr).unwrap();
                self.connection.udp = Some(endpoint);
                log::info!("Connection by udp on port {}", udp_port);
                self.node.signals().send(ProxyEvent::HelloUdp(0));
            }
            LoginStatus::InvalidPlayerName => {
                log::warn!("Invalid character name {}", character);
            }
            LoginStatus::AlreadyLogged => {
                log::warn!("Character name '{}' already logged", character);
            }
            LoginStatus::PlayerLimit => {
                log::error!("Server full");
            }
        }
        callback(ServerEvent::LoginStatus(status));
    }

    fn process_hello_udp(&mut self, attempt: usize, callback: impl Fn(ServerEvent)) {
        if !self.connection.has_udp_hasdshake {
            match self.connection.session_token {
                Some(token) => match self.connection.udp {
                    Some(udp_endpoint) => {
                        if attempt < UDP_HANDSHAKE_MAX_ATTEMPS {
                            log::trace!("Udp handshake attempt: {}", attempt);
                            self.send_to_server(udp_endpoint, ClientMessage::ConnectUdp(token));
                            let next_time = (attempt * attempt) as u64 + 1;
                            let next_message_timer = Duration::from_millis(next_time);
                            let hello_udp = ProxyEvent::HelloUdp(attempt + 1);
                            self.node.signals().send_with_timer(hello_udp, next_message_timer);
                        }
                        else {
                            log::warn!("Unable to communicate by udp.");
                            callback(ServerEvent::UdpReachable(false));
                        }
                    }
                    None => log::warn!("Attempt to send hello udp without known endpoint"),
                },
                None => log::warn!("Attempt to send hello udp without logged session"),
            }
        }
    }

    fn process_udp_connected(&mut self, callback: impl Fn(ServerEvent)) {
        let tcp = *self.connection.tcp.as_ref().unwrap();
        self.send_to_server(tcp, ClientMessage::TrustUdp);
        self.connection.has_udp_hasdshake = true;
        log::info!("Client udp successful reachable from server");
        callback(ServerEvent::UdpReachable(true));
    }

    fn process_finish_game(&mut self, callback: impl Fn(ServerEvent)) {
        self.connection.has_udp_hasdshake = false;
        callback(ServerEvent::FinishGame);
    }
}
