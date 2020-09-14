use crate::message::{ClientMessage, ServerMessage, LoginStatus, ServerInfo};
use crate::version::{self, Compatibility};
use crate::util::{self};

use message_io::events::{EventQueue};
use message_io::network::{NetworkManager, NetEvent, Endpoint};

use std::net::{IpAddr, SocketAddr};
use std::io::{self, BufRead};
use std::time::{Duration};

#[derive(Debug)]
pub enum ClosingReason {
    //Finished,
    Forced,
    ConnectionLost,
    IncompatibleVersions,
    ServerFull,
}

#[derive(Debug)]
enum Event {
    Network(NetEvent<ServerMessage>),
    Login,
    WaitingArena(Duration),
    Close(ClosingReason),
}

struct ConnectionInfo {
    ip: IpAddr,
    udp_port: Option<u16>,
    tcp: Endpoint,
    udp: Option<Endpoint>,
    udp_confirmed: bool,
    session_token: Option<usize>,
}

pub struct ClientManager {
    event_queue: EventQueue<Event>,
    network: NetworkManager,
    player_name: Option<String>,
    server_info: Option<ServerInfo>,
    connection: ConnectionInfo,
}

impl ClientManager {
    pub fn new(addr: SocketAddr, player_name: Option<&str>) -> Option<ClientManager> {
        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();
        let mut network = NetworkManager::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let network_sender = event_queue.sender().clone();
        ctrlc::set_handler(move || network_sender.send_with_priority(Event::Close(ClosingReason::Forced))).unwrap();

        match network.connect_tcp(addr) {
            Ok(tcp_endpoint) => {
                log::info!("Connected to server by tcp on {}", addr);
                println!("Connect to server!");
                Some(ClientManager {
                    event_queue,
                    network,
                    player_name: player_name.map(|s| s.to_string()),
                    server_info: None,
                    connection: ConnectionInfo {
                        ip: addr.ip(),
                        udp_port: None,
                        tcp: tcp_endpoint,
                        udp: None,
                        udp_confirmed: false,
                        session_token: None,
                    },

                })
            },
            Err(_) => {
                log::error!("Could not connect to server by tcp on {}", addr);
                println!("Could not connect to server on {}", addr);
                None
            }
        }
    }

    pub fn run(&mut self) -> ClosingReason {
        self.network.send(self.connection.tcp, ClientMessage::Version(version::current().to_string())).unwrap();
        loop {
            let event = self.event_queue.receive();
            log::trace!("[Process event] - {:?}", event);
            match event {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(endpoint, message) => {
                        log::trace!("Message from {}", endpoint.addr());
                        match message {
                            ServerMessage::Version(server_version, server_side_compatibility) => {
                                self.process_version(&server_version, server_side_compatibility);
                            }
                            ServerMessage::ServerInfo(info) => {
                                self.process_server_info(info);
                            }
                            ServerMessage::LoginStatus(status) => {
                                self.process_login_status(status);
                            }
                            ServerMessage::PlayerListUpdated(players) => {
                                self.process_notify_new_player(players);
                            }
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
                                self.process_step();
                            },
                        }
                    },
                    NetEvent::AddedEndpoint(_) => unreachable!(),
                    NetEvent::RemovedEndpoint(_) => {
                        println!("Connection lost with the server");
                        self.event_queue.sender().send_with_priority(Event::Close(ClosingReason::ConnectionLost))
                    }
                },
                Event::Login => {
                    self.process_login();
                },
                Event::WaitingArena(duration) => {
                    self.process_waiting_arena(duration);
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
            Compatibility::Fully => {
                log::trace!("Fully compatible versions {}", version::current());
                println!("Server version: {} (same version)", server_version);
            }
            Compatibility::OkOutdated => {
                log::warn!("Compatible server version but differs. Client: {}. Server: {}", version::current(), server_version);
                println!("Compatible versions but it is recomendable to update. Client: {}. Server: {}", version::current(), server_version);
            }
            Compatibility::None => {
                log::error!("Incompatible server version. Client: {}. Server: {}", version::current(), server_version);
                eprintln!("Incompatible server version. Client: {}. Server: {}. Aborted", version::current(), server_version);
                self.event_queue.sender().send_with_priority(Event::Close(ClosingReason::IncompatibleVersions));
            }
        }
        if compatibility.is_compatible() {
            self.network.send(self.connection.tcp, ClientMessage::RequestServerInfo).unwrap();
        }
    }

    fn process_server_info(&mut self, info: ServerInfo) {
        log::info!("Server info: {:?}", info);
        println!("Game info:");
        println!(" - Current players: {} ({} of {})", util::format::player_names(&info.logged_players), info.logged_players.len(), info.players_number);
        println!(" - Winner points: {}", info.winner_points);
        println!(" - Map size: {}x{}", info.map_size, info.map_size);

        self.connection.udp_port = Some(info.udp_port);
        self.server_info = Some(info);
        self.event_queue.sender().send(Event::Login);
    }

    fn process_login_status(&mut self, status: LoginStatus) {
        let player_name = self.player_name.as_ref().unwrap();
        match status {
            LoginStatus::Logged(token) => {
                log::info!("Logged with name '{}' successful", player_name);
                println!("Logged!");
                self.connection.session_token = Some(token);
            },
            LoginStatus::Reconnected(token) => {
                log::info!("Reconnected with name '{}' successful", player_name);
                println!("Reconnected!");
                self.connection.session_token = Some(token);
            },
            LoginStatus::InvalidPlayerName => {
                log::warn!("Invalid character name {}", player_name);
                self.player_name = None;
                self.event_queue.sender().send(Event::Login);
            },
            LoginStatus::AlreadyLogged => {
                log::warn!("Character name '{}' already logged", player_name);
                println!("Character name already logged, please use another name");
                self.player_name = None;
                self.event_queue.sender().send(Event::Login);
            },
            LoginStatus::PlayerLimit => {
                log::error!("Server full");
                println!("Player limit reached: {}, Try later :(" , self.server_info.as_ref().unwrap().players_number);
                self.event_queue.sender().send_with_priority(Event::Close(ClosingReason::ServerFull));
            },
        }
    }

    fn process_notify_new_player(&mut self, player_names: Vec<String>) {
        let mut info = self.server_info.as_mut().unwrap();
        info.logged_players = player_names;
        log::info!("Player list updated: {}", util::format::player_names(&info.logged_players));
        println!("Player list updated: {} ({} of {})", util::format::player_names(&info.logged_players), info.logged_players.len(), info.players_number);
    }

    fn process_login(&mut self) {
        if self.player_name.is_none() {
            println!("Choose a character (an unique letter from A to Z): ");
            let possible_name = io::stdin().lock().lines().next().unwrap().unwrap();
            if util::is_valid_player_name(&possible_name) {
                self.player_name = Some(possible_name);
            }
            else {
                println!("Character name '{}' not valid, try again", possible_name);
                log::warn!("Character name '{}' not valid", possible_name);
                return self.event_queue.sender().send(Event::Login);
            }
        }

        let name = self.player_name.clone().unwrap().clone();
        self.network.send(self.connection.tcp, ClientMessage::Login(name)).unwrap();
    }

    fn process_start_game(&mut self) {
        log::info!("Start game");
        println!("Players ready!");
    }

    fn process_finish_game(&mut self) {
        log::info!("Finish game");
        println!("DEBUG: Finish game");
        // In this state, the client needs to login again if want to continue playing
    }

    fn process_prepare_arena(&mut self, duration: Duration) {
        log::info!("The arena will be start in {}", duration.as_secs_f32());
        println!("Initializing arena in");
        self.event_queue.sender().send_with_priority(Event::WaitingArena(duration));
    }

    fn process_waiting_arena(&mut self, duration: Duration) {
        println!("{}...", duration.as_secs_f32());
        let interval = Duration::from_secs(1);
        if duration > interval {
            let remaining_time = match duration.subsec_millis() {
                0 => duration - interval,
                _ => Duration::from_secs(duration.as_secs()),
            };
            self.event_queue.sender().send_with_timer(Event::WaitingArena(remaining_time), interval);
        }
    }

    fn process_start_arena(&mut self) {
        log::info!("Start arena");
        println!("DEBUG: Start arena");
    }

    fn process_finish_arena(&mut self) {
        log::info!("Finish arena");
        println!("DEBUG: Finish arena");
    }

    fn process_step(&mut self) {
        log::info!("Process step");
        println!("DEBUG: Step");
    }
}


