use super::connection::{ServerConnection, ConnectionResult, ServerEvent};
use super::util::store::{Store};
use super::actions::{ActionManager, Action, ActionableEvent, ClosingReason};
use super::state::{State};

use super::frontend::{Frontend, Viewport, Renderer, Input};

use message_io::events::{EventQueue};

use std::net::{SocketAddr};

#[derive(Debug)]
pub enum AppEvent<I> {
    Server(ServerEvent),
    Frontend(I),
    Close(ClosingReason),
}

pub struct Application<F: Frontend> {
    event_queue: EventQueue<AppEvent<<F::Input as Input>::Event>>,
    store: Store<ActionManager>,
    server: ServerConnection,
    viewport: F::Viewport,
    input: F::Input,
}

impl<F: Frontend> Application<F> {
    pub fn new(server_addr: SocketAddr, player_name: Option<&str>) -> Application<F> {
        let mut event_queue = EventQueue::new();

        let internal_actionable_sender = event_queue.sender().map(&|actionable_event| {
            match actionable_event {
                ActionableEvent::Api(api_call) => AppEvent::Server(ServerEvent::Api(api_call)),
                ActionableEvent::Close(reason) => AppEvent::Close(reason),
            }
        });

        let state = State::new(server_addr, player_name);
        let actions = ActionManager::new(internal_actionable_sender);
        let store = Store::new(state, actions);

        let internal_server_sender = event_queue.sender().map(&|internal_event| {
            AppEvent::Server(ServerEvent::Internal(internal_event))
        });

        let internal_input_sender = event_queue.sender().map(&|internal_event| {
            AppEvent::Frontend(internal_event)
        });

        Application {
            event_queue,
            server: ServerConnection::new(store.clone(), internal_server_sender),
            viewport: F::Viewport::new_full_screen(),
            input: F::Input::new(store.clone(), internal_input_sender),
            store,
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
                AppEvent::Frontend(input_event) => {
                    self.input.process_event(input_event);
                }
                AppEvent::Close(reason) => {
                    log::info!("Closing client. Reason: {:?}", reason);
                    break reason
                },
            }
        }
    }
}
