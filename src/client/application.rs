use super::connection::{ServerProxy};
use super::util::store::{Store};
use super::actions::{ActionManager, Action, Dispatcher, Closer, ClosingReason};
use super::state::{State};

use super::frontend::{Frontend, Renderer};

use message_io::events::{EventSender, EventQueue};

use std::net::{SocketAddr};
use std::time::{Duration};

#[derive(Debug)]
pub enum AppEvent {
    Action(Action),
    Close(ClosingReason),
    Draw,
}

pub struct Application<F: Frontend> {
    event_queue: EventQueue<AppEvent>,
    store: Store<ActionManager>,
    _server: ServerProxy, // Kept because we need its internal thread running
    _input: F::Input, // Kept because we need its internal thread running
}

impl<F: Frontend> Application<F> {
    pub fn new(server_addr: SocketAddr, player_name: Option<&str>) -> Application<F> {
        let mut event_queue = EventQueue::new();

        let action_dispatcher = ActionDispatcher { sender: event_queue.sender().clone() };
        let mut server = ServerProxy::new(action_dispatcher.clone());

        let state = State::new(server_addr, player_name);
        let closer = AppCloser { sender: event_queue.sender().clone() };
        let actions = ActionManager::new(closer, server.api());

        Application {
            event_queue,
            store: Store::new(state, actions),
            _server: server,
            _input: F::init_input(action_dispatcher.clone()),
        }
    }

    pub fn run(&mut self) -> ClosingReason {
        let mut renderer = F::init_renderer();
        self.store.dispatch(Action::StartApp);
        self.event_queue.sender().send(AppEvent::Draw);

        loop {
            let event = self.event_queue.receive();
            log::trace!("[Process event] - {:?}", event);
            match event {
                AppEvent::Action(action) => {
                    self.store.dispatch(action);
                },
                AppEvent::Draw => {
                    renderer.render(&self.store.state_manager());
                    self.event_queue.sender().send_with_timer(AppEvent::Draw, Duration::from_millis(1000));
                },
                AppEvent::Close(reason) => {
                    log::info!("Closing client. Reason: {:?}", reason);
                    break reason
                },
            }
        }
    }
}

#[derive(Clone)]
pub struct ActionDispatcher {
    sender: EventSender<AppEvent>
}

impl Dispatcher for ActionDispatcher {
    fn dispatch(&mut self, action: Action) {
        self.sender.send(AppEvent::Action(action));
    }
}

pub struct AppCloser {
    sender: EventSender<AppEvent>
}

impl Closer for AppCloser {
    fn close(&mut self, reason: ClosingReason) {
        self.sender.send_with_priority(AppEvent::Close(reason));
    }
}
