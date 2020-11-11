use super::session::{Room, SessionCreationResult};
use super::game::{Game};

use crate::message::{ClientMessage, ServerMessage, ServerInfo,
    LoginStatus, LoggedKind, SessionToken};
use crate::version::{self, Compatibility};
use crate::util::{self};

use message_io::events::{EventQueue};
use message_io::network::{NetworkManager, NetEvent, Endpoint};

use itertools::{Itertools};

use std::time::{Duration, Instant};
use std::collections::{HashSet};

lazy_static! {
    static ref GAME_STEP_DURATION: Duration = Duration::from_secs_f32(1.0 / 30.0);
}

#[derive(Debug)]
enum Event {
    Network(NetEvent<ClientMessage>),
    CreateGame,
    PrepareArena,
    StartArena,
    GameStep,
    Reset,
    Close,
}

pub struct Config {
    pub tcp_port: u16,
    pub udp_port: u16,
    pub players_number: u8,
    pub map_size: usize,
    pub winner_points: usize,
    pub arena_waiting: Duration,
}

pub struct ServerManager {
    event_queue: EventQueue<Event>,
    network: NetworkManager,
    server_info_subscriptions: HashSet<Endpoint>,
    room: Room<Endpoint>,
    game: Option<Game>,
    timestamp_last_arena_creation: Option<Instant>,
    config: Config,
}

impl ServerManager {
    pub fn new(config: Config) -> Option<ServerManager> {
        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();
        let mut network = NetworkManager::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let network_sender = event_queue.sender().clone();
        ctrlc::set_handler(move || network_sender.send_with_priority(Event::Close)).unwrap();

        let network_interface = "0.0.0.0";
        if let Err(_) = network.listen_tcp((network_interface, config.tcp_port)) {
            log::error!("Can not run server on tcp port {}", config.tcp_port);
            return None;
        }

        if let Err(_) = network.listen_udp((network_interface, config.udp_port)) {
            log::error!("Can not run server on udp port {}", config.udp_port);
            return None;
        }

        log::info!("Server running on tcp ports {} (tcp) and {} (udp) for {} players", config.tcp_port, config.udp_port, config.players_number);
        Some(ServerManager {
            event_queue,
            network,
            server_info_subscriptions: HashSet::new(),
            room: Room::new(config.players_number as usize),
            game: None,
            timestamp_last_arena_creation: None,
            config,
        })
    }

    pub fn run(&mut self) {
        loop {
            let event = self.event_queue.receive();
            log::trace!("[Process event] - {:?}", event);
            match event {
                Event::Network(net_event) => match net_event {
                    NetEvent::AddedEndpoint(_) => (),
                    NetEvent::RemovedEndpoint(endpoint) => {
                        self.process_disconnection(endpoint);
                    },
                    NetEvent::Message(endpoint, message) => {
                        log::trace!("Message from {}", endpoint.addr());
                        match message {
                            ClientMessage::Version(client_version) => {
                                self.process_version(endpoint, &client_version);
                            },
                            ClientMessage::SubscribeServerInfo => {
                                self.process_subscribe_server_info(endpoint);
                            },
                            ClientMessage::Login(character) => {
                                self.process_login(endpoint, character);
                            },
                            ClientMessage::Logout => {
                                self.process_logout(endpoint);
                            },
                            ClientMessage::ConnectUdp(session_token) => {
                                self.process_connect_udp(endpoint, session_token);
                            },
                            ClientMessage::TrustUdp => {
                                self.process_trust_udp(endpoint);
                            },
                            ClientMessage::MovePlayer => {
                                //TODO
                            },
                            ClientMessage::CastSkill => {
                                //TODO
                            },
                        }
                    },
                },
                Event::CreateGame => {
                    self.process_create_game();
                },
                Event::PrepareArena => {
                    self.process_prepare_arena();
                },
                Event::StartArena => {
                    self.process_start_arena();
                },
                Event::GameStep => {
                    self.process_game_step();
                },
                Event::Reset => {
                    self.process_reset();
                }
                Event::Close => {
                    log::info!("Closing server");
                    break
                },
            }
        }
    }

    fn process_version(&mut self, endpoint: Endpoint, client_version: &str) {
        let compatibility = version::check(&client_version, version::current());
        match compatibility {
            Compatibility::Fully =>
                log::trace!("Fully compatible versions: {}", client_version),
            Compatibility::NotExact =>
                log::warn!("Compatible client version, but not exact. Client: {}. Server: {}", client_version, version::current()),
            Compatibility::None =>
                log::error!("Incompatible client version. Client: {}. Server: {}. Rejected", client_version, version::current()),
        };

        self.network.send(endpoint, ServerMessage::Version(version::current().into(), compatibility)).unwrap();
        if let Compatibility::None = compatibility {
            self.network.remove_resource(endpoint.resource_id()).unwrap();
        }
    }

    fn process_subscribe_server_info(&mut self, endpoint: Endpoint) {
        let info = ServerInfo {
            udp_port: self.config.udp_port,
            players_number: self.config.players_number,
            map_size: self.config.map_size as u16,
            winner_points: self.config.winner_points as u16,
            logged_players: self.room.sessions().map(|session| session.character()).collect(),
        };

        log::trace!("Client {} has subscribed to server info", endpoint.addr());
        self.server_info_subscriptions.insert(endpoint);
        self.network.send(endpoint, ServerMessage::ServerInfo(info)).unwrap();
    }

    fn process_login(&mut self, endpoint: Endpoint, character: char) {
        let status =
        if !util::is_valid_character(character) {
            log::warn!("Invalid character name '{}' has tried to login", character);
            LoginStatus::InvalidPlayerName
        }
        else {
            match self.room.create_session(character, endpoint) {
                SessionCreationResult::Created(token) => {
                    let characters = self.room.sessions().map(|session| session.character()).sorted();
                    log::info!("New player logged: {}, current players: {}", character, util::format::character_list(characters));
                    LoginStatus::Logged(token, LoggedKind::FirstTime)
                },
                SessionCreationResult::Recycled(token) => {
                    log::info!("Player '{}' reconnected", character);
                    LoginStatus::Logged(token, LoggedKind::Reconnection)
                },
                SessionCreationResult::AlreadyLogged => {
                    log::warn!("Player '{}' has tried to login but the character name is already logged", character);
                    LoginStatus::AlreadyLogged
                },
                SessionCreationResult::Full => {
                    log::warn!("Player '{}' has tried to login but the player limit has been reached", character);
                    LoginStatus::PlayerLimit
                },
            }
        };

        log::trace!("{} with player name '{}' attempts to login. Status: {:?}", endpoint.addr(), character, status);
        self.network.send(endpoint, ServerMessage::LoginStatus(character, status)).unwrap();

        if let LoginStatus::Logged(_, kind) = status { // First time connection
            match kind {
                LoggedKind::FirstTime => {
                    let characters = self.room.sessions().map(|session| session.character()).collect();
                    self.network.send_all(self.server_info_subscriptions.iter(), ServerMessage::DynamicServerInfo(characters)).ok();

                    if self.game.is_none() && self.room.is_full() {
                        self.event_queue.sender().send(Event::CreateGame);
                    }
                },
                LoggedKind::Reconnection => {
                    if let Some(game) = &self.game {
                        self.network.send(endpoint, ServerMessage::StartGame).ok();

                        let timestamp = self.timestamp_last_arena_creation.as_ref().unwrap();
                        let duration_since_arena_creation = Instant::now().duration_since(*timestamp);
                        if let Some(waiting) = self.config.arena_waiting.checked_sub(duration_since_arena_creation) {
                            self.network.send(endpoint, ServerMessage::PrepareArena(waiting)).ok();
                        }

                        if let Some(arena) = game.arena() {
                            self.network.send(endpoint, ServerMessage::StartArena(arena.number())).ok();
                        }
                    }
                }
            }
        }
    }

    fn process_logout(&mut self, endpoint: Endpoint) {
        if self.game.is_some() {
            if let Some(session) = self.room.session_by_endpoint_mut(endpoint) {
                session.disconnect();
                log::info!("Player '{}' disconnected", session.character());
            }
        }
        else {
            if let Some(session) = self.room.remove_session_by_endpoint(endpoint) {
                log::info!("Player '{}' logout, current players: {} ", session.character(),
                    util::format::character_list(self.room.sessions().map(|session| session.character()).sorted()));

                let characters = self.room.sessions().map(|session| session.character()).collect();
                self.network.send_all(self.server_info_subscriptions.iter(), ServerMessage::DynamicServerInfo(characters)).ok();
            }
        }
    }

    fn process_connect_udp(&mut self, udp_endpoint: Endpoint, session_token: SessionToken) {
        match self.room.session_mut(session_token) {
            Some(session) => {
                log::trace!("Attached udp endpoint to session '{}'", session_token);
                session.set_untrusted_fast_endpoint(udp_endpoint);
                self.network.send(udp_endpoint, ServerMessage::UdpConnected).unwrap();
            }
            None => log::warn!("Attempt to attach udp endpoint to non-existent session '{}'", session_token),
        }
    }

    fn process_trust_udp(&mut self, related_tcp_endpoint: Endpoint) {
        match self.room.session_by_endpoint_mut(related_tcp_endpoint) {
            Some(session) => match session.trust_in_fast_endpoint() {
                Some(_) => log::trace!("Trusted udp endpoint for session '{}'", session.token()),
                None => log::error!("Attempt to trust into a non-existent udp endpoint. Session '{}'", session.token()),
            },
            None => log::error!("Attempt to trust an udp endpoint in an non-existent session"),
        }
    }

    fn process_create_game(&mut self) {
        log::info!("Starting new game");
        let characters = self.room.sessions().map(|session| session.character());
        let game = Game::new(characters, self.config.winner_points);
        self.game = Some(game);

        self.network.send_all(self.room.safe_endpoints(), ServerMessage::StartGame).ok();

        self.event_queue.sender().send(Event::PrepareArena);
    }

    fn process_prepare_arena(&mut self) {
        log::trace!("Initializing next arena in {} seconds...", self.config.arena_waiting.as_secs_f32());

        self.network.send_all(self.room.safe_endpoints(), ServerMessage::PrepareArena(self.config.arena_waiting)).ok();

        self.event_queue.sender().send_with_timer(Event::StartArena, self.config.arena_waiting);
        self.timestamp_last_arena_creation = Some(Instant::now());
    }

    fn process_start_arena(&mut self) {
        let game = self.game.as_mut().unwrap();
        let arena = game.create_new_arena();
        log::info!("Start arena {}", arena.number());

        self.network.send_all(self.room.safe_endpoints(), ServerMessage::StartArena(arena.number())).ok();

        self.event_queue.sender().send(Event::GameStep);
    }

    fn process_game_step(&mut self) {
        let game = self.game.as_mut().unwrap();

        log::trace!("Processing step");
        game.step();

        let arena = game.arena().unwrap();
        self.network.send_all(self.room.faster_endpoints(), ServerMessage::Step).ok();

        if arena.has_finished() {
            log::info!("End arena {}. Raking: {}", arena.number(), util::format::character_list(arena.ranking().clone()));
            log::info!("Game points: {}", util::format::character_points_list(game.pole()));
            self.network.send_all(self.room.safe_endpoints(), ServerMessage::FinishArena).ok();

            if game.has_finished() {
                log::info!("End game");
                self.network.send_all(self.room.safe_endpoints(), ServerMessage::FinishGame).ok();

                self.event_queue.sender().send(Event::Reset);
            }
            else {
                self.event_queue.sender().send(Event::PrepareArena);
            }
        }
        else {
            self.event_queue.sender().send_with_timer(Event::GameStep, *GAME_STEP_DURATION);
        }
    }

    fn process_reset(&mut self) {
        log::info!("Reset server");
        self.game = None;
        self.room.clear();

        let characters = self.room.sessions().map(|session| session.character()).collect();
        self.network.send_all(self.server_info_subscriptions.iter(), ServerMessage::DynamicServerInfo(characters)).ok();
    }

    fn process_disconnection(&mut self, endpoint: Endpoint) {
        if self.server_info_subscriptions.remove(&endpoint) {
            log::trace!("Client {} has unsubscribed to server info", endpoint.addr());
        }
        self.process_logout(endpoint);
    }
}

