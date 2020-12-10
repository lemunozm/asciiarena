use crate::message::{LoginStatus, ServerInfo, ClientMessage, ServerMessage,
    LoggedKind, GameInfo, ArenaInfo, Frame, ArenaChange};
use crate::version::{self, Compatibility};
use crate::direction::{Direction};
use crate::ids::{SkillId};

use message_io::events::{EventQueue, EventSender};
use message_io::network::{Network, NetEvent, Endpoint};

use std::net::{IpAddr, SocketAddr};
use std::thread::{self, JoinHandle};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
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
    ServerInfo(ServerInfo),
    PlayerListUpdated(Vec<char>),
    LoginStatus(LoginStatus),
    UdpReachable(bool),
    StartGame(GameInfo),
    FinishGame,
    WaitArena(Duration),
    StartArena(ArenaInfo),
    ArenaChange(ArenaChange),
    ArenaStep(Frame),
    FinishArena,
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
enum Event {
    Api(ApiCall),
    Network(NetEvent<ServerMessage>),
    HelloUdp(usize),
}

pub struct ServerProxy {
    event_sender: EventSender<Event>,
    proxy_thread_running: Arc<AtomicBool>,
    proxy_thread_handle: Option<JoinHandle<()>>,
}

impl ServerProxy {
    pub fn new(event_callback: impl Fn(ServerEvent) + Send + 'static) -> ServerProxy {
        let mut event_queue = EventQueue::new();
        let event_sender = event_queue.sender().clone();

        let proxy_thread_running = Arc::new(AtomicBool::new(true));
        let proxy_thread_handle = {
            let running = proxy_thread_running.clone();
            thread::Builder::new()
                .name("asciiarena: server event collector".into())
                .spawn(move || {
                let sender = event_queue.sender().clone();
                let mut connection = ServerConnection::new(sender, event_callback);
                while running.load(Ordering::Relaxed) {
                    if let Some(event) = event_queue.receive_timeout(*EVENT_SAMPLING_TIMEOUT) {
                        connection.process_event(event);
                    }
                }
            })
        }.unwrap();

        ServerProxy {
            event_sender,
            proxy_thread_running,
            proxy_thread_handle: Some(proxy_thread_handle),
        }
    }

    pub fn api(&mut self) -> ServerApi {
        return ServerApi {
            proxy_thread_running: self.proxy_thread_running.clone(),
            sender: self.event_sender.clone(),
        }
    }
}

impl Drop for ServerProxy {
    fn drop(&mut self) {
        self.proxy_thread_running.store(false, Ordering::Relaxed);
        self.proxy_thread_handle.take().unwrap().join().unwrap();
    }
}

pub struct ServerApi {
    proxy_thread_running: Arc<AtomicBool>,
    sender: EventSender<Event>,
}

impl ServerApi {
    pub fn call(&mut self, api_call: ApiCall) {
        if self.proxy_thread_running.load(Ordering::Relaxed) {
            // Only send the event if the server proxy thread is running
            self.sender.send(Event::Api(api_call));
        }
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

struct ServerConnection<C> {
    event_sender: EventSender<Event>,
    network: Network,
    connection: ConnectionInfo,
    event_callback: C,
}

impl<C> ServerConnection<C>
where C: Fn(ServerEvent) {
    pub fn new(event_sender: EventSender<Event>, event_callback: C) -> ServerConnection<C> {
        let sender = event_sender.clone();
        let network = Network::new(move |net_event| sender.send(Event::Network(net_event)));

        ServerConnection {
            event_sender,
            network,
            connection: ConnectionInfo {
                ip: None,
                udp_port: None,
                tcp: None,
                udp: None,
                has_udp_hasdshake: false,
                session_token: None,
            },
            event_callback
        }
    }

    pub fn connect(&mut self, addr: SocketAddr) -> ConnectionStatus {
        self.disconnect(); // Ensure there is no connection, reset if there is.
        match self.network.connect_tcp(addr) {
            Ok(tcp_endpoint) => {
                log::info!("Connected to server by tcp on {}", addr);
                self.connection.tcp = Some(tcp_endpoint);
                self.connection.ip = Some(addr.ip());
                ConnectionStatus::Connected
            },
            Err(_) => {
                log::error!("Could not connect to server by tcp on {}", addr);
                ConnectionStatus::NotFound
            }
        }
    }

    pub fn disconnect(&mut self) -> ConnectionStatus {
        self.connection.has_udp_hasdshake = false;
        self.connection.session_token = None;
        self.connection.udp_port = None;
        self.connection.udp = None;
        self.connection.ip = None;
        if let Some(endpoint) = self.connection.tcp {
            self.network.remove_resource(endpoint.resource_id());
            self.connection.tcp = None;
        }
        ConnectionStatus::NotConnected
    }

    pub fn logout(&mut self) {
        self.connection.has_udp_hasdshake = false;
        self.connection.session_token = None;
        self.connection.udp = None;
        let tcp = *self.connection.tcp.as_ref().unwrap();
        self.network.send(tcp, ClientMessage::Logout);
    }

    pub fn process_event(&mut self, event: Event) {
        log::trace!("Process event: {:?}", event);
        match event {
            Event::Api(api_event) => {
                match api_event {
                    ApiCall::Connect(addr) => {
                        let result = self.connect(addr);
                        (self.event_callback)(ServerEvent::ConnectionResult(result));
                    },
                    ApiCall::Disconnect => {
                        let result = self.disconnect();
                        (self.event_callback)(ServerEvent::ConnectionResult(result));
                    },
                    ApiCall::CheckVersion(version) => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.network.send(tcp, ClientMessage::Version(version));
                    },
                    ApiCall::SubscribeInfo => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.network.send(tcp, ClientMessage::SubscribeServerInfo);
                    },
                    ApiCall::Login(character) => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.network.send(tcp, ClientMessage::Login(character));
                    },
                    ApiCall::Logout => {
                        self.logout()
                    },
                    ApiCall::MovePlayer(direction) => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.network.send(tcp, ClientMessage::MovePlayer(direction));
                    },
                    ApiCall::CastSkill(direction, id) => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.network.send(tcp, ClientMessage::CastSkill(direction, id));
                    },
                }
            },
            Event::Network(net_event) => match net_event {
                NetEvent::Message(_, message) => match message {
                    ServerMessage::Version(server_version, server_side_compatibility) => {
                        self.process_version(server_version, server_side_compatibility);
                    },
                    ServerMessage::ServerInfo(info) => {
                        self.process_server_info(info);
                    },
                    ServerMessage::DynamicServerInfo(players) => {
                        (self.event_callback)(ServerEvent::PlayerListUpdated(players));
                    },
                    ServerMessage::LoginStatus(character, status) => {
                        self.process_login_status(character, status);
                    },
                    ServerMessage::UdpConnected => {
                        self.process_udp_connected();
                    },
                    ServerMessage::StartGame(game_info) => {
                        (self.event_callback)(ServerEvent::StartGame(game_info));
                    },
                    ServerMessage::FinishGame => {
                        self.process_finish_game();
                    },
                    ServerMessage::WaitArena(duration) => {
                        (self.event_callback)(ServerEvent::WaitArena(duration));
                    },
                    ServerMessage::StartArena(arena_info) => {
                        (self.event_callback)(ServerEvent::StartArena(arena_info));
                    },
                    ServerMessage::ArenaChange(arena_change) => {
                        (self.event_callback)(ServerEvent::ArenaChange(arena_change));
                    },
                    ServerMessage::FinishArena => {
                        (self.event_callback)(ServerEvent::FinishArena);
                    },
                    ServerMessage::Step(frame) => {
                        (self.event_callback)(ServerEvent::ArenaStep(frame));
                    },
                },
                NetEvent::AddedEndpoint(_) => unreachable!(),
                NetEvent::RemovedEndpoint(_) => {
                    let result = ConnectionStatus::Lost;
                    (self.event_callback)(ServerEvent::ConnectionResult(result));
                },
                NetEvent::DeserializationError(endpoint) => {
                    log::error!(
                        "Server sends an unknown message. Connection rejected. \
                        Ensure the version compatibility.",
                    );
                    self.network.remove_resource(endpoint.resource_id()).unwrap();
                }
            },
            Event::HelloUdp(attempt) => {
                self.process_hello_udp(attempt);
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
            Compatibility::NotExact => {
                log::warn!("Compatible server version, but not exact. Client: {}. Server: {}", version::current(), server_version);
            }
            Compatibility::None => {
                log::error!("Incompatible server version. Client: {}. Server: {}", version::current(), server_version);
            }
        }

        (self.event_callback)(ServerEvent::CheckedVersion(server_version, compatibility));
    }

    fn process_server_info(&mut self, info: ServerInfo) {
        self.connection.udp_port = Some(info.udp_port);
        (self.event_callback)(ServerEvent::ServerInfo(info));
    }

    fn process_login_status(&mut self, character: char, status: LoginStatus) {
        match status {
            LoginStatus::Logged(token, kind) => {
                let kind_str = match kind {
                    LoggedKind::FirstTime => "Logged",
                    LoggedKind::Reconnection => "Reconnected",
                };
                log::info!(
                    "{} with name '{}' successful. Token Id: {}",
                    kind_str, character, token
                );

                let udp_port = *self.connection.udp_port.as_ref().unwrap();
                let ip = *self.connection.ip.as_ref().unwrap();
                self.connection.session_token = Some(token);
                self.connection.udp = Some(self.network.connect_udp((ip, udp_port)).unwrap());
                log::info!("Connection by udp on port {}", udp_port);
                self.event_sender.send(Event::HelloUdp(0));
            },
            LoginStatus::InvalidPlayerName => {
                log::warn!("Invalid character name {}", character);
            },
            LoginStatus::AlreadyLogged => {
                log::warn!("Character name '{}' already logged", character);
            },
            LoginStatus::PlayerLimit => {
                log::error!("Server full");
            },
        }
        (self.event_callback)(ServerEvent::LoginStatus(status));
    }

    fn process_hello_udp(&mut self, attempt: usize) {
        if !self.connection.has_udp_hasdshake {
            match self.connection.session_token {
                Some(token) => match self.connection.udp {
                    Some(udp_endpoint) =>
                        if attempt < UDP_HANDSHAKE_MAX_ATTEMPS {
                            log::trace!("Udp handshake attempt: {}", attempt);
                            self.network.send(udp_endpoint, ClientMessage::ConnectUdp(token));
                            let next_time = (attempt * attempt) as u64 + 1;
                            let next_message_timer = Duration::from_millis(next_time);
                            let hello_udp = Event::HelloUdp(attempt + 1);
                            self.event_sender.send_with_timer(hello_udp, next_message_timer);
                        }
                        else {
                            log::warn!("Unable to communicate by udp.");
                            (self.event_callback)(ServerEvent::UdpReachable(false));
                        }
                    None => log::warn!("Attempt to send hello udp without known endpoint"),
                },
                None => log::warn!("Attempt to send hello udp without logged session"),
            }
        }
    }

    fn process_udp_connected(&mut self) {
        let tcp = *self.connection.tcp.as_ref().unwrap();
        self.network.send(tcp, ClientMessage::TrustUdp);
        self.connection.has_udp_hasdshake = true;
        log::info!("Client udp successful reachable from server");
        (self.event_callback)(ServerEvent::UdpReachable(true));
    }

    fn process_finish_game(&mut self) {
        self.connection.has_udp_hasdshake = false;
        (self.event_callback)(ServerEvent::FinishGame);
    }

}

