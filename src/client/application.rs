use super::connection::{ServerConnection, ConnectionResult};
use super::events::{AppEvent, ServerEvent, ClosingReason};
use super::util::store::{Store};
use super::actions::{ActionManager, Action};
use super::state::{State};

use message_io::events::{EventQueue};

use std::net::{SocketAddr};

pub struct Application {
    event_queue: EventQueue<AppEvent>,
    store: Store<ActionManager>,
    server: ServerConnection,
}

impl Application {
    pub fn new(server_addr: SocketAddr, player_name: Option<&str>) -> Application {
        let mut event_queue = EventQueue::new();

        // build store
        let state = State::new(server_addr, player_name);
        let actions = ActionManager::new(event_queue.sender().clone());
        let store = Store::new(state, actions);

        // build server
        let internal_server_sender = event_queue.sender().map(&|internal_event| {
            AppEvent::Server(ServerEvent::Internal(internal_event))
        });
        let server = ServerConnection::new(store.clone(), internal_server_sender);

        Application {
            event_queue,
            store,
            server,
        }
    }

    pub fn run(&mut self) -> ClosingReason {
        let server_addr = self.store.state().server().addr();
        match self.server.connect(server_addr) {
            ConnectionResult::Connected => self.store.dispatch(Action::Connected),
            ConnectionResult::NotFound => return ClosingReason::ServerNotFound(server_addr),
        }

        loop {
            let event = self.event_queue.receive();
            log::trace!("[Process event] - {:?}", event);
            match event {
                AppEvent::Server(server_event) => {
                    self.server.process_event(server_event);
                },
                AppEvent::Close(reason) => {
                    log::info!("Closing client. Reason: {:?}", reason);
                    break reason
                },
            }
        }
    }
}
