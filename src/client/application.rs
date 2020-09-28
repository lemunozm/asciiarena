use super::connection::{ServerProxy};
use super::util::store::{Store};
use super::actions::{ActionManager, Action, Dispatcher, Closer, ClosingReason};
use super::state::{State};

use super::frontend::{Frontend, Viewport, Renderer, Input};

use message_io::events::{EventSender, EventQueue, Senderable};

use std::net::{SocketAddr};

#[derive(Debug)]
pub enum AppEvent {
    Action(Action),
    Close(ClosingReason),
    Draw,
}

pub struct Application<F: Frontend> {
    event_queue: EventQueue<AppEvent>,
    store: Store<ActionManager>,
    server: ServerProxy,
    viewport: F::Viewport,
    input: F::Input,
}

impl<F: Frontend> Application<F> {
    pub fn new(server_addr: SocketAddr, player_name: Option<&str>) -> Application<F> {
        let mut event_queue = EventQueue::new();

        let action_dispatcher = ActionDispatcher::new(event_queue.sender().clone());
        let mut server = ServerProxy::new(action_dispatcher.clone());

        let state = State::new(server_addr, player_name);
        let closer = AppCloser::new(event_queue.sender().clone());
        let actions = ActionManager::new(closer, server.api());

        Application {
            event_queue,
            store: Store::new(state, actions),
            server,
            viewport: F::Viewport::new_full_screen(),
            input: F::Input::new(action_dispatcher.clone()),
        }
    }

    pub fn run(&mut self) -> ClosingReason {
        self.store.dispatch(Action::StartApp);

        loop {
            let event = self.event_queue.receive();
            log::trace!("[Process event] - {:?}", event);
            match event {
                AppEvent::Action(action) => {
                    self.store.dispatch(action);
                },
                AppEvent::Draw => {
                    //render
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
    event_sender: EventSender<AppEvent>
}

impl ActionDispatcher {
    fn new(event_sender: EventSender<AppEvent>) -> ActionDispatcher {
        ActionDispatcher { event_sender }
    }
}

impl Dispatcher for ActionDispatcher {
    fn dispatch(&mut self, action: Action) {
        self.event_sender.send(AppEvent::Action(action));
    }
}

pub struct AppCloser {
    event_sender: EventSender<AppEvent>
}

impl AppCloser {
    fn new(event_sender: EventSender<AppEvent>) -> AppCloser {
        AppCloser { event_sender }
    }
}

impl Closer for AppCloser {
    fn close(&mut self, reason: ClosingReason) {
        self.event_sender.send(AppEvent::Close(reason));
    }
}
