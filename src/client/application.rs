use super::connection::{ServerConnection, ServerEvent, ConnectionResult, LoginStatus};
use super::store::{Store, state::State, actions::{ActionManager, Action}};

use message_io::events::{EventQueue, Senderable};

use std::net::{SocketAddr};

#[derive(Debug)]
pub enum ClosingReason {
    ServerNotFound(SocketAddr),
    //Forced, //Ctrl-C
    ConnectionLost,
    IncompatibleVersions,
}

#[derive(Debug)]
enum Event {
    Server(ServerEvent),
    //FrontEnd(F::FrontendEvent),
    Close(ClosingReason),
}

pub struct Application {
    event_queue: EventQueue<Event>,
    server: ServerConnection,
    store: Store<ActionManager>,
    addr: SocketAddr,
    player_name: Option<String>,
}

impl Application {
    pub fn new(addr: SocketAddr, player_name: Option<&str>) -> Application {
        let mut event_queue = EventQueue::new();

        let sender = event_queue.sender().map(&|event| Event::Server(event));
        let server = ServerConnection::new(addr, sender);

        let actions = ActionManager::new(0, 0);

        Application {
            event_queue,
            server,
            store: Store::new(State::new(addr), actions),
            addr,
            player_name: player_name.map(|n| n.into()),
        }
    }

    pub fn run(&mut self) -> ClosingReason {
        match self.server.connect(self.addr) {
            ConnectionResult::Connected => self.server.check_version(),
            ConnectionResult::NotFound => {
                return ClosingReason::ServerNotFound(self.addr)
            }
        }

        loop {
            let event = self.event_queue.receive();
            log::trace!("[Process event] - {:?}", event);
            match event {
                Event::Server(server_event) => {
                    self.process_server_event(server_event);
                },
                //Event::FrontEnd => { },
                Event::Close(reason) => {
                    log::info!("Closing client. Reason: {:?}", reason);
                    break reason
                },
            }
        }
    }

    pub fn process_server_event(&mut self, event: ServerEvent) {
        match event {
            ServerEvent::Internal(internal) => {
                self.server.process_internal_event(internal);
            },
            ServerEvent::Disconnected => {
                self.close(ClosingReason::ConnectionLost);
            },
            ServerEvent::CheckedVersion(server_version, compatibility) => {
                self.store.dispatch(Action::SetVersionInfo(server_version, compatibility));
                self.store.state().player_name();

                if compatibility.is_compatible() {
                    self.server.subscribe_info();
                }
                else {
                    self.close(ClosingReason::IncompatibleVersions);
                }
            },
            ServerEvent::ServerInfo(_info) => {
                self.login()
            },
            ServerEvent::PlayerListUpdated(_players) => {
            },
            ServerEvent::LoginStatus(_player_name, status) => {
                match status {
                    LoginStatus::Logged(_token, _kind) => {
                    },
                    LoginStatus::InvalidPlayerName => {
                    },
                    LoginStatus::AlreadyLogged => {
                    },
                    LoginStatus::PlayerLimit => {
                    },
                }
            },
            ServerEvent::UdpReachable => {
            },
            ServerEvent::StartGame => {
            },
            ServerEvent::FinishGame => {
            },
            ServerEvent::PrepareArena(_duration) => {
            },
            ServerEvent::StartArena => {
            },
            ServerEvent::FinishArena => {
            },
            ServerEvent::ArenaStep => {
                println!("step")
            },
        }
    }

    fn login(&mut self) {
        //ensure to have a name in player_name at this point
        let name = self.player_name.clone().unwrap().clone();
        self.server.login(name);
    }

    fn close(&mut self, reason: ClosingReason) {
        self.event_queue.sender().send_with_priority(Event::Close(reason))
    }
}
