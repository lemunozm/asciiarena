use crate::message::{self, ClientMessage, ServerMessage, LoginStatus};
use crate::version::{self, Compatibility};
use crate::util::{self};

use message_io::events::{EventQueue};
use message_io::network::{NetworkManager, NetEvent, TransportProtocol, Endpoint};

use std::net::{IpAddr, SocketAddr};
use std::io::{self, BufRead};

#[derive(Debug)]
pub enum ClosingReason {
    //Finished,
    Forced,
    ConnectionLost,
    IncompatibleVersions,
}

#[derive(Debug)]
pub enum LoginPhase {
    Initial,
    Logged,
    //UdpHandshake{msg_count: usize},
}

#[derive(Debug)]
enum Event {
    Network(NetEvent<ServerMessage>),
    Login(LoginPhase),
    Close(ClosingReason),
}

struct ServerInfo {
    ip: IpAddr,
    udp_port: Option<u16>,
    tcp_endpoint: Endpoint,
    udp_endpoint: Option<Endpoint>,
    udp_confirmed: bool,
    session_token: Option<usize>,
}

pub struct ClientManager {
    event_queue: EventQueue<Event>,
    network: NetworkManager,
    player_name: Option<String>,
    server: ServerInfo,
}

impl ClientManager {
    pub fn new(addr: SocketAddr, player_name: Option<&str>) -> Option<ClientManager> {
        let mut event_queue = EventQueue::new();

        let network_sender = event_queue.sender().clone();
        let mut network = NetworkManager::new(move |net_event| network_sender.send(Event::Network(net_event)));

        let network_sender = event_queue.sender().clone();
        ctrlc::set_handler(move || network_sender.send_with_priority(Event::Close(ClosingReason::Forced))).unwrap();

        if let Some((tcp_endpoint, _)) = network.connect(addr, TransportProtocol::Tcp) {
            let msg = format!("Connected to server on '{}' by tcp", addr);
            log::info!("{}", msg);
            println!("{}", msg);
            Some(ClientManager {
                event_queue,
                network,
                player_name: player_name.map(|s| s.to_string()),
                server: ServerInfo {
                    ip: addr.ip(),
                    udp_port: None,
                    tcp_endpoint,
                    udp_endpoint: None,
                    udp_confirmed: false,
                    session_token: None,
                }

            })
        }
        else {
            let msg = format!("Could not connect to server on '{}' by tcp", addr);
            log::error!("{}", msg);
            eprintln!("{}", msg);
            None
        }
    }

    pub fn run(&mut self) -> ClosingReason {
        self.network.send(self.server.tcp_endpoint, ClientMessage::Version(version::current().to_string()));
        loop {
            let event = self.event_queue.receive();
            log::trace!("[Process event] - {:?}", event);
            match event {
                Event::Network(net_event) => match net_event {
                    NetEvent::Message(endpoint, message) => {
                        log::trace!("Message from {}", self.network.endpoint_remote_address(endpoint).unwrap());
                        match message {
                            ServerMessage::Version(server_version, server_side_compatibility) => {
                                self.process_msg_version(&server_version, server_side_compatibility);
                            }
                            ServerMessage::ServerInfo(info) => {
                                self.process_msg_server_info(&info);
                            }
                            ServerMessage::LoginStatus(status) => {
                                self.process_msg_login_status(&status);
                            }
                            ServerMessage::NotifyNewPlayer(name) => {
                                self.process_msg_notify_new_player(&name);
                            }
                        }
                    },
                    NetEvent::AddedEndpoint(_, _) => unreachable!(),
                    NetEvent::RemovedEndpoint(_) => {
                        println!("Connection lost with the server");
                        self.event_queue.sender().send_with_priority(Event::Close(ClosingReason::ConnectionLost))
                    }
                },
                Event::Login(phase) => {
                    self.process_login(phase);
                }
                Event::Close(reason) => {
                    log::info!("Closing client. Reason: {:?}", reason);
                    break reason
                }
            }
        }
    }

    fn process_msg_version(&mut self, server_version: &str, server_side_compatibility: Compatibility) {
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
            self.network.send(self.server.tcp_endpoint, ClientMessage::RequestServerInfo);
        }
    }

    fn process_msg_server_info(&mut self, info: &message::ServerInfo) {
        self.server.udp_port = Some(info.udp_port);
        if info.players as usize > info.logged_players.len() {
            self.event_queue.sender().send(Event::Login(LoginPhase::Initial));
        }
    }

    fn process_msg_login_status(&mut self, status: &LoginStatus) {
        //TODO
    }

    fn process_msg_notify_new_player(&mut self, name: &str) {
        //TODO
    }

    fn process_login(&mut self, phase: LoginPhase) {
        match phase {
            LoginPhase::Initial => {
                if self.player_name.is_none() {
                    loop {
                        println!("Choose a character (an unique letter from A to Z): ");
                        let possible_name = io::stdin().lock().lines().next().unwrap().unwrap();
                        if util::is_valid_player_name(&possible_name) {
                            self.player_name = Some(possible_name);
                            break;
                        }
                    }
                }
                //Check name
                let name = self.player_name.clone().unwrap().clone();
                self.network.send(self.server.tcp_endpoint, ClientMessage::Login(name));
            },
            LoginPhase::Logged => {
            },
                //TODO
                /*
                let (endpoint, _) = self.network.connect(SocketAddr::from((self.server.ip, self.server.udp_port)), TransportProtocol::Udp).unwrap();
                self.server.udp_endpoint = Some(endpoint);
                self.event_queue.sender().send(Event::Login(LoginPhase::UdpHandshake{msg_count: 0}));
                */
            /*
            LoginPhase::UdpHandshake{msg_count} => {
                if msg_count <
                self.network.send()
                self.event_queue.sender().send_with_timer(Event::Login(LoginPhase::UdpHandshake{msg_count: msg_count + 1}), Duration::from_millis());
            },
            */
        }
        //Create socket
        //User name by CLI if not setted.
    }
}


