use super::session::{RoomSession, SessionStatus};
use super::game::{Game};

use crate::message::{ClientMessage, ServerMessage, ServerInfo, GameInfo, ArenaInfo,
    LoginStatus, LoggedKind, SessionToken, EntityData, Frame, ArenaChange};
use crate::version::{self, Compatibility};
use crate::direction::{Direction};
use crate::util::{self};

use message_io::events::{EventQueue};
use message_io::network::{Network, NetEvent, Endpoint};

use itertools::{Itertools};

use std::time::{Duration, Instant};
use std::collections::{HashSet};

lazy_static! {
    static ref GAME_STEP_DURATION: Duration = Duration::from_secs_f32(1.0 / 30.0);
}

#[derive(Debug)]
enum Event {
    Network(NetEvent<ClientMessage>),
    AsyncCreateGame, // Could take time in processing
    AsyncStartArena, // Generated Eventually
    GameStep,        // Generated Eventually
    Close,           // Main loop control
}

pub struct Config {
    pub tcp_port: u16,
    pub udp_port: u16,
    pub players_number: u8,
    pub map_size: usize,
    pub winner_points: usize,
    pub arena_waiting: Duration,
}

pub struct ServerManager<'a> {
    config: &'a Config,
    network: Network,
    subscriptions: HashSet<Endpoint>,
    room: RoomSession<Endpoint, char>,
    game: Option<Game>,
    timestamp_last_arena_creation: Option<Instant>,
    event_queue: EventQueue<Event>,
}

impl<'a> ServerManager<'a> {
    pub fn new(config: &'a Config) -> Option<ServerManager<'a>> {
        let mut event_queue = EventQueue::new();

        let sender = event_queue.sender().clone();
        let mut network = Network::new(move |net_event| sender.send(Event::Network(net_event)));

        let signal_sender = event_queue.sender().clone();
        ctrlc::set_handler(move || {
            signal_sender.send_with_priority(Event::Close)
        }).unwrap();

        let network_interface = "0.0.0.0";
        if let Err(_) = network.listen_tcp((network_interface, config.tcp_port)) {
            log::error!("Can not run server on TCP port {}", config.tcp_port);
            return None;
        }

        if let Err(_) = network.listen_udp((network_interface, config.udp_port)) {
            log::error!("Can not run server on UDP port {}", config.udp_port);
            return None;
        }

        log::info!(
            "Server running on ports {} (tcp) and {} (udp) for {} players",
            config.tcp_port,
            config.udp_port,
            config.players_number
        );

        Some(ServerManager {
            event_queue,
            network,
            subscriptions: HashSet::new(),
            room: RoomSession::new(config.players_number as usize),
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
                Event::AsyncCreateGame => {
                    self.process_create_game();
                },
                Event::AsyncStartArena => {
                    self.process_start_arena();
                },
                Event::GameStep => {
                    self.process_game_step();
                },
                Event::Close => {
                    log::info!("Closing server");
                    break
                },
                Event::Network(net_event) => match net_event {
                    NetEvent::AddedEndpoint(endpoint) => {
                        log::trace!("{} has connected", endpoint);
                    },
                    NetEvent::RemovedEndpoint(endpoint) => {
                        log::trace!("{} has disconnected", endpoint);
                        self.process_disconnection(endpoint);
                    },
                    NetEvent::DeserializationError(endpoint) => {
                        log::error!("{} sends an unknown message. Connection rejected", endpoint);
                        self.network.remove_resource(endpoint.resource_id()).unwrap();
                    }
                    NetEvent::Message(endpoint, message) => {
                        log::trace!("Message from {}", endpoint.addr());
                        match message {
                            ClientMessage::Version(client_version) => {
                                self.process_version(endpoint, &client_version);
                            },
                            ClientMessage::SubscribeServerInfo => {
                                self.process_subscribe_server_info(endpoint);
                            },
                            ClientMessage::Login(user) => {
                                self.process_login(endpoint, user);
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
                            ClientMessage::MovePlayer(direction) => {
                                self.process_move_player(endpoint, direction);
                            },
                            ClientMessage::CastSkill => {
                                //TODO
                            },
                        }
                    },
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
                log::warn!(
                    "Compatible client version, but not exact. Client: {}. Server: {}",
                    client_version,
                    version::current()
                ),
            Compatibility::None =>
                log::warn!(
                    "Incompatible client version. Client: {}. Server: {}. Connection rejected",
                    client_version,
                    version::current()
                ),
        };

        let message = ServerMessage::Version(version::current().into(), compatibility);
        self.network.send(endpoint, message);

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
            logged_players: self.room
                .sessions()
                .map(|session| *session.user())
                .collect(),
        };

        log::trace!("Client {} has subscribed to server info", endpoint.addr());
        self.subscriptions.insert(endpoint);
        self.network.send(endpoint, ServerMessage::ServerInfo(info));
    }

    fn process_login(&mut self, endpoint: Endpoint, player_symbol: char) {
        let status =
        if !util::is_valid_character(player_symbol) {
            log::warn!("Invalid character symbol '{}' has tried to login", player_symbol);
            LoginStatus::InvalidPlayerName
        }
        else {
            match self.room.create_session(player_symbol, endpoint) {
                SessionStatus::Created(token) => {
                    let player_symbols = self.room
                        .sessions()
                        .map(|session| session.user())
                        .sorted();

                    log::info!(
                        "New player logged: {}, current players: {}",
                        player_symbol,
                        util::format::items_to_string(player_symbols)
                    );
                    LoginStatus::Logged(token, LoggedKind::FirstTime)
                },
                SessionStatus::Recycled(token) => {
                    log::info!("Player '{}' reconnected", player_symbol);
                    LoginStatus::Logged(token, LoggedKind::Reconnection)
                },
                SessionStatus::AlreadyLogged => {
                    log::warn!(
                        "Player '{}' has tried to login but the character symbol is already logged",
                        player_symbol
                    );
                    LoginStatus::AlreadyLogged
                },
                SessionStatus::Full => {
                    log::warn!(
                        "Player '{}' has tried to login but the player limit has been reached",
                        player_symbol
                    );
                    LoginStatus::PlayerLimit
                },
            }
        };

        log::trace!(
            "{} with player '{}' attempts to login. Status: {:?}",
            endpoint.addr(),
            player_symbol,
            status
        );

        self.network.send(endpoint, ServerMessage::LoginStatus(player_symbol, status));

        if let LoginStatus::Logged(_, kind) = status {
            match kind {
                LoggedKind::FirstTime => {
                    let player_symbols = self.room
                        .sessions()
                        .map(|session| *session.user())
                        .collect();

                    let message = ServerMessage::DynamicServerInfo(player_symbols);
                    self.network.send_all(self.subscriptions.iter(), message);

                    if self.game.is_none() && self.room.is_full() {
                        self.event_queue.sender().send(Event::AsyncCreateGame);
                    }
                },
                LoggedKind::Reconnection => {
                    if let Some(game) = &self.game {
                        let message = Self::create_start_game_message(game);
                        self.network.send(endpoint, message);

                        let timestamp = self.timestamp_last_arena_creation.as_ref().unwrap();
                        let duration = Instant::now().duration_since(*timestamp);
                        if let Some(waiting) = self.config.arena_waiting.checked_sub(duration) {
                            self.network.send(endpoint, ServerMessage::WaitArena(waiting));
                        }

                        if let Some(_) = game.arena() {
                            let message = Self::create_start_arena_message(game);
                            self.network.send(endpoint, message);
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
                log::info!("Player '{}' disconnected", session.user());
            }
        }
        else {
            if let Some(session) = self.room.remove_session_by_endpoint(endpoint) {
                let player_symbols = self.room
                    .sessions()
                    .map(|session| *session.user())
                    .collect::<Vec<_>>();

                log::info!(
                    "Player '{}' logout, current players: {} ",
                    session.user(),
                    util::format::items_to_string(player_symbols.iter().sorted())
                );

                let message = ServerMessage::DynamicServerInfo(player_symbols);
                self.network.send_all(self.subscriptions.iter(), message);
            }
        }
    }

    fn process_connect_udp(&mut self, udp_endpoint: Endpoint, session_token: SessionToken) {
        match self.room.session_mut(session_token) {
            Some(session) => {
                log::trace!("Attached udp endpoint to session '{}'", session_token);
                session.set_untrusted_fast_endpoint(udp_endpoint);
                self.network.send(udp_endpoint, ServerMessage::UdpConnected);
            }
            None =>
                log::warn!(
                    "Attempt to attach udp endpoint to non-existent session '{}'",
                    session_token
                )
        }
    }

    fn process_trust_udp(&mut self, related_tcp_endpoint: Endpoint) {
        match self.room.session_by_endpoint_mut(related_tcp_endpoint) {
            Some(session) => match session.trust_in_fast_endpoint() {
                Some(_) =>
                    log::trace!(
                        "Trusted udp endpoint for session '{}'",
                        session.token()
                    ),
                None =>
                    log::error!(
                        "Attempt to trust into a non-existent udp endpoint. Session '{}'",
                        session.token()
                    ),
            },
            None => log::error!("Attempt to trust an udp endpoint in an non-existent session"),
        }
    }

    fn process_create_game(&mut self) {
        log::info!("Starting new game");
        let player_symbols = self.room.sessions().map(|session| *session.user());
        let game = Game::new(
            self.config.map_size,
            self.config.winner_points,
            player_symbols
        );

        let message = Self::create_start_game_message(&game);
        self.network.send_all(self.room.safe_endpoints(), message);

        self.game = Some(game);
        self.process_wait_arena();
    }

    fn process_wait_arena(&mut self) {
        log::trace!(
            "Initializing next arena in {} seconds...",
            self.config.arena_waiting.as_secs_f32()
        );

        let message = ServerMessage::WaitArena(self.config.arena_waiting);
        self.network.send_all(self.room.safe_endpoints(), message);

        self.event_queue
            .sender()
            .send_with_timer(Event::AsyncStartArena, self.config.arena_waiting);

        self.timestamp_last_arena_creation = Some(Instant::now());
    }

    fn process_start_arena(&mut self) {
        self.game.as_mut().unwrap().create_new_arena();
        let game = self.game.as_ref().unwrap();
        let arena = game.arena().unwrap();
        log::info!("Start arena {}", game.arena_number());

        let entities = arena.entities();
        let player_positions = game
            .players()
            .iter()
            .map(|(symbol, player)| {
                let entity = player
                    .control()
                    .borrow()
                    .entity_id()
                    .map(|id| entities.get(&id).unwrap());

                let position = match entity {
                    Some(entity) => entity.position().to_string(),
                    None => "-".into(),
                };
                (symbol, position)
            })
            .collect::<Vec<_>>();

        log::trace!("Player positions: {}", util::format::pair_items_to_string(player_positions));

        let message = Self::create_start_arena_message(game);
        self.network.send_all(self.room.safe_endpoints(), message);

        self.event_queue.sender().send(Event::GameStep);
    }

    fn process_finish_arena(&mut self) {
        let game = self.game.as_mut().unwrap();
        let player_partial_points_pairs = game
            .pole()
            .iter()
            .map(|player| (player.character().symbol(), player.partial_points()))
            .collect::<Vec<_>>();

        log::info!(
            "End arena {}. Raking: {}",
            game.arena_number(),
            util::format::pair_items_to_string(player_partial_points_pairs)
        );

        let player_total_points_pairs = game
            .pole()
            .iter()
            .map(|player| (player.character().symbol(), player.total_points()))
            .collect::<Vec<_>>();

        log::info!(
            "Game points: {}",
            util::format::pair_items_to_string(player_total_points_pairs)
        );

        self.network.send_all(self.room.safe_endpoints(), ServerMessage::FinishArena);

        if game.has_finished() {
            log::info!("End game");
            self.network.send_all(self.room.safe_endpoints(), ServerMessage::FinishGame);

            self.process_reset();
        }
        else {
            self.process_wait_arena();
        }
    }

    fn process_game_step(&mut self) {
        let game = self.game.as_mut().unwrap();

        log::trace!("Processing step");

        let previous_players = game.living_players().len();
        game.step();
        let current_players = game.living_players().len();

        if current_players < previous_players {
            let points = game
                .players()
                .values()
                .map(|player| player.partial_points())
                .collect();

            let change = ArenaChange::PlayerPartialPoints(points);
            let message = ServerMessage::ArenaChange(change);
            self.network.send_all(self.room.faster_endpoints(), message);
        }

        if current_players > 1 {
            let arena = game.arena().unwrap();
            let entities = arena.entities().values().map(|entity| {
                EntityData {
                    id: entity.id(),
                    character_id: entity.character().id(),
                    position: entity.position(),
                    live: entity.live(),
                    energy: entity.live(),
                }
            }).collect();

            let message = ServerMessage::Step(Frame { entities });
            self.network.send_all(self.room.faster_endpoints(), message);

            self.event_queue.sender().send_with_timer(Event::GameStep, *GAME_STEP_DURATION);
        }
        else {
            self.process_finish_arena();
        }
    }

    fn process_move_player(&mut self, endpoint: Endpoint, direction: Direction) {
        match self.game.as_mut() {
            Some(game) => match self.room.session_by_endpoint(endpoint) {
                Some(session) => {
                    let player = game.player_mut(*session.user()).expect("Exists");
                    player.walk(direction);
                }
                None => log::warn!("Unlogged client attempted to move a character. Maybe an attack?")
            },
            None => log::warn!("Client attempted to move a character without a created game")
        };
    }

    fn process_reset(&mut self) {
        log::info!("Reset server");
        self.game = None;
        self.room.clear();

        let player_symbols = self.room
            .sessions()
            .map(|session| *session.user())
            .collect();

        let message = ServerMessage::DynamicServerInfo(player_symbols);
        self.network.send_all(self.subscriptions.iter(), message);
    }

    fn process_disconnection(&mut self, endpoint: Endpoint) {
        if self.subscriptions.remove(&endpoint) {
            log::trace!("Client {} has unsubscribed to server info", endpoint.addr());
        }
        self.process_logout(endpoint);
    }

    fn create_start_game_message(game: &Game) -> ServerMessage {
        let game_info = GameInfo {
            characters: game.characters()
                .iter()
                .map(|(_, character)| (**character).clone())
                .collect(),
            players: game.players()
                .iter()
                .map(|(_, player)| (
                    player.character().id(),
                    player.total_points()
                ))
                .collect()
        };

        ServerMessage::StartGame(game_info)
    }

    fn create_start_arena_message(game: &Game) -> ServerMessage {
        let arena_info = ArenaInfo {
            number: game.arena_number(),
            players: game.players()
                .iter()
                .map(|(_, player)| (
                    player.control().borrow().entity_id(),
                    player.partial_points()
                ))
                .collect()
        };

        ServerMessage::StartArena(arena_info)
    }
}

