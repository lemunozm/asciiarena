pub use crate::message::{LoginStatus, ServerInfo};
use crate::message::{ClientMessage, ServerMessage, LoggedKind};
use crate::version::{self, Compatibility};
use crate::util::{self};

use super::actions::{Action, ServerApi, ApiCall, Dispatcher};
use super::state::{ConnectionStatus};

use message_io::events::{EventQueue, EventSender};
use message_io::network::{NetworkManager, NetEvent, Endpoint};

use std::net::{IpAddr, SocketAddr};
use std::thread::{self, JoinHandle};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::{Duration};

const UDP_HANDSHAKE_MAX_ATTEMPS: usize = 10;
const EVENT_SAMPLING_TIMEOUT: u64 = 50; //ms

#[derive(Debug)]
pub enum InternalEvent {
}

#[derive(Debug)]
pub enum ServerEvent {
    Api(ApiCall),
    Network(NetEvent<ServerMessage>),
    HelloUdp(usize),
}

pub struct ServerProxy {
    proxy_thread_running: Arc<AtomicBool>,
    proxy_thread_handle: Option<JoinHandle<()>>,
    event_sender: EventSender<ServerEvent>,
}

impl ServerProxy {
    pub fn new(actions: impl Dispatcher + 'static) -> ServerProxy {
        let mut event_queue = EventQueue::new();
        let event_sender = event_queue.sender().clone();
        let internal_event_sender = event_queue.sender().clone();

        let proxy_thread_running = Arc::new(AtomicBool::new(true));
        let proxy_thread_handle = {
            let running = proxy_thread_running.clone();
            let timeout = Duration::from_millis(EVENT_SAMPLING_TIMEOUT);
            let mut server_connection = ServerConnection::new(internal_event_sender, actions);
            thread::Builder::new().name("asciiarena: server event collector".into()).spawn(move || {
                while running.load(Ordering::Relaxed) {
                    if let Some(event) = event_queue.receive_event_timeout(timeout) {
                        server_connection.process_event(event);
                    }
                }
            })
        }.unwrap();

        ServerProxy {
            proxy_thread_running,
            proxy_thread_handle: Some(proxy_thread_handle),
            event_sender,
        }
    }

    pub fn api(&mut self) -> impl ServerApi {
        return ServerApiImpl { sender: self.event_sender.clone() }
    }
}

impl Drop for ServerProxy {
    fn drop(&mut self) {
        self.proxy_thread_running.store(false, Ordering::Relaxed);
        self.proxy_thread_handle.take().unwrap().join().unwrap();
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
    event_sender: EventSender<ServerEvent>,
    network: NetworkManager,
    connection: ConnectionInfo,
    actions: Box<dyn Dispatcher>,
}

impl ServerConnection {
    pub fn new(event_sender: EventSender<ServerEvent>, actions: impl Dispatcher + 'static) -> ServerConnection {
        let network_sender = event_sender.clone();
        let network = NetworkManager::new(move |net_event| {
            network_sender.send(ServerEvent::Network(net_event))
        });

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
            actions: Box::new(actions),
        }
    }

    pub fn connect(&mut self, addr: SocketAddr) -> ConnectionStatus {
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
        if let Some(endpoint) = self.connection.tcp {
            self.network.remove_resource(endpoint.resource_id());
        }
        ConnectionStatus::NotConnected
    }

    pub fn process_event(&mut self, event: ServerEvent) {
        match event {
            ServerEvent::Api(api_event) => {
                match api_event {
                    ApiCall::Connect(addr) => {
                        let result = self.connect(addr);
                        self.actions.dispatch(Action::ConnectionResult(result));
                    },
                    ApiCall::Disconnect => {
                        let result = self.disconnect();
                        self.actions.dispatch(Action::ConnectionResult(result));
                    },
                    ApiCall::CheckVersion(version) => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.network.send(tcp, ClientMessage::Version(version)).unwrap();
                    },
                    ApiCall::SubscribeInfo => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.network.send(tcp, ClientMessage::SubscribeServerInfo).unwrap();
                    },
                    ApiCall::Login(player_name) => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.network.send(tcp, ClientMessage::Login(player_name)).unwrap();
                    },
                    ApiCall::Logout => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.network.send(tcp, ClientMessage::Logout).unwrap();
                    },
                    ApiCall::MovePlayer => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.network.send(tcp, ClientMessage::Move).unwrap();
                    },
                    ApiCall::CastSkill => {
                        let tcp = *self.connection.tcp.as_ref().unwrap();
                        self.network.send(tcp, ClientMessage::Skill).unwrap();
                    },
                }
            },
            ServerEvent::Network(net_event) => match net_event {
                NetEvent::Message(_, message) => match message {
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
                },
                NetEvent::AddedEndpoint(_) => unreachable!(),
                NetEvent::RemovedEndpoint(_) => {
                    self.actions.dispatch(Action::ConnectionResult(ConnectionStatus::Lost));
                },
            },
            ServerEvent::HelloUdp(attempt) => {
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

        self.actions.dispatch(Action::CheckedVersion(server_version, compatibility));
    }

    fn process_server_info(&mut self, info: ServerInfo) {
        log::info!("Server info: {:?}", info);
        self.connection.udp_port = Some(info.udp_port);
        self.actions.dispatch(Action::ServerInfo(info));
    }

    fn process_dynamic_server_info(&mut self, player_names: Vec<String>) {
        log::info!("Player list updated: {}", util::format::player_names(&player_names));
        self.actions.dispatch(Action::PlayerListUpdated(player_names));
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
                let ip = *self.connection.ip.as_ref().unwrap();
                self.connection.session_token = Some(token);
                self.connection.udp = Some(self.network.connect_udp((ip, udp_port)).unwrap());
                log::info!("Connection by udp on port {}", udp_port);
                self.event_sender.send(ServerEvent::HelloUdp(0));
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
        self.actions.dispatch(Action::LoginStatus(status));
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
                            self.event_sender.send_with_timer(ServerEvent::HelloUdp(attempt + 1), next_message_timer);
                        }
                        else {
                            log::warn!("Unable to communicate by udp.");
                            self.actions.dispatch(Action::UdpReachable(false));
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
        log::info!("Udp successful reachable");
        self.actions.dispatch(Action::UdpReachable(true));
    }

    fn process_start_game(&mut self) {
        log::info!("Start game");
        self.actions.dispatch(Action::StartGame);
    }

    fn process_finish_game(&mut self) {
        log::info!("Finish game");
        self.connection.has_udp_hasdshake = false;
        self.actions.dispatch(Action::FinishGame);
    }

    fn process_prepare_arena(&mut self, duration: Duration) {
        log::info!("The arena will be start in {}", duration.as_secs_f32());
        self.actions.dispatch(Action::PrepareArena(duration));
    }

    fn process_start_arena(&mut self) {
        log::info!("Start arena");
        self.actions.dispatch(Action::StartArena);
    }

    fn process_finish_arena(&mut self) {
        log::info!("Finish arena");
        self.actions.dispatch(Action::FinishArena);
    }

    fn process_arena_step(&mut self) {
        log::info!("Process arena step");
        self.actions.dispatch(Action::ArenaStep);
    }
}

struct ServerApiImpl {
    sender: EventSender<ServerEvent>,
}

impl ServerApi for ServerApiImpl {
    fn call(&mut self, api_call: ApiCall) {
        self.sender.send(ServerEvent::Api(api_call));
    }
}
