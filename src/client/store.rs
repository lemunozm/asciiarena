use super::state::{State, StaticGameInfo, VersionInfo, GameStatus, Arena, ArenaStatus};
use super::server_proxy::{ServerApi, ApiCall, ConnectionStatus, ServerEvent};

use crate::version::{self};
use crate::direction::{Direction};

use std::net::{SocketAddr};
use std::time::{Instant};
use std::collections::{HashMap};

/// Action API
#[derive(Debug)]
pub enum Action {
    StartApp,
    Connect(SocketAddr),
    Disconnect,
    Login(char),
    Logout,
    CloseGame,
    CloseApp,
    MovePlayer(Direction),
    ServerEvent(ServerEvent),
}

pub struct Store {
    state: State,
    server: ServerApi,
    close : bool,
}

impl Store {
    pub fn new(state: State, server: ServerApi) -> Store {
        Store {
            state,
            server,
            close: false,
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn should_close(&self) -> bool {
        self.close
    }

    pub fn dispatch(&mut self, action: Action) {
        log::trace!("Dispatch: {:?}", action);
        match action {
            Action::StartApp => {
                if let Some(addr) = self.state.server.addr {
                    self.server.call(ApiCall::Connect(addr));
                }
            },

            Action::Connect(addr) => {
                self.state.server.addr = Some(addr);
                self.server.call(ApiCall::Connect(addr));
            },

            Action::Disconnect => {
                self.state.server.addr = None;
                self.server.call(ApiCall::Disconnect);
            }

            Action::Login(character) => {
                self.state.user.character = Some(character);
                self.server.call(ApiCall::Login(character));
            },

            Action::Logout => {
                self.state.user.character = None;
                self.state.user.login_status = None;
                self.server.call(ApiCall::Logout);
            }

            Action::CloseGame => {
                self.state.server.game.status = GameStatus::NotStarted;
                self.state.server.game.arena = None;
            }

            Action::CloseApp => {
                self.close = true;
            }

            Action::MovePlayer(direction) => {
                self.server.call(ApiCall::MovePlayer(direction));
            }

            Action::ServerEvent(server_event) => match server_event {
                ServerEvent::ConnectionResult(status)  => {
                    self.state.server.connection_status = status;
                    if let ConnectionStatus::Connected = status {
                        self.server.call(ApiCall::CheckVersion(version::current().into()));
                    }
                    else {
                        self.dispatch(Action::ServerEvent(ServerEvent::FinishGame));
                        self.state.server.logged_players = Vec::new();
                        self.state.server.game.arena = None;
                        self.state.server.game_info = None;
                    }
                },

                ServerEvent::CheckedVersion(server_version, compatibility) => {
                    let version_info = VersionInfo { version: server_version, compatibility };
                    self.state.server.version_info = Some(version_info);

                    if compatibility.is_compatible() {
                        self.server.call(ApiCall::SubscribeInfo);
                    }
                    else {
                        // Protect the client against an unknown or not compatible server version
                        self.server.call(ApiCall::Disconnect);
                    }
                },

                ServerEvent::ServerInfo(info) => {
                    let game_info = StaticGameInfo {
                        players_number: info.players_number as usize,
                        map_size: info.map_size as usize,
                        winner_points: info.winner_points as usize,
                    };
                    self.state.server.udp_port = Some(info.udp_port);
                    self.state.server.game_info = Some(game_info);
                    self.state.server.logged_players = info.logged_players;

                    if let Some(character) = self.state.user.character {
                        self.server.call(ApiCall::Login(character));
                    }
                },

                ServerEvent::PlayerListUpdated(player_names) => {
                    self.state.server.logged_players = player_names;
                },

                ServerEvent::LoginStatus(status) => {
                    self.state.user.login_status = Some(status);
                },

                ServerEvent::UdpReachable(value) => {
                    self.state.server.udp_confirmed = Some(value);
                },

                ServerEvent::StartGame(game_info) => {
                    self.state.server.game.status = GameStatus::Started;
                    self.state.server.game.characters = game_info.characters
                        .into_iter()
                        .map(|character| (character.id(), character))
                        .collect();
                },

                ServerEvent::FinishGame => {
                    self.state.server.game.status = GameStatus::Finished;
                    self.state.server.udp_confirmed = None;
                    self.state.user.character = None;
                    self.state.user.login_status = None;
                },

                ServerEvent::WaitArena(duration) => {
                    self.state.server.game.next_arena_timestamp = Some(
                        Instant::now() + duration
                    );
                },

                ServerEvent::StartArena(arena_info) => {
                    self.state.server.game.next_arena_timestamp = None;
                    self.state.server.game.arena_number = arena_info.number;
                    self.state.server.game.players = arena_info.players
                        .into_iter()
                        .collect();

                    self.state.server.game.arena = Some(Arena {
                        status: ArenaStatus::Playing,
                        entities: HashMap::new(),
                    });
                },

                ServerEvent::FinishArena => {
                    self.state.server.game.arena_mut().status = ArenaStatus::Finished;
                },

                ServerEvent::ArenaStep(frame) => {
                    let entities_map = frame.entities
                        .into_iter()
                        .map(|entity| (entity.id, entity))
                        .collect::<HashMap<_, _>>();

                    // If the entity no longer exists, remove it from players
                    for player in &mut self.state.server.game.players {
                        if let Some(entity_id) = &mut player.1 {
                            if !entities_map.contains_key(&entity_id) {
                                player.1 = None;
                            }
                        }
                    }

                    self.state.server.game.arena_mut().entities = entities_map;
                },
            },
        }
    }
}
