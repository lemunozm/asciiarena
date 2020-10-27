use super::server_proxy::{ServerProxy};
use super::util::store::{Store};
use super::actions::{ActionManager, Action, Dispatcher, AppController};
use super::state::{State};
pub use super::state::{Config};

use super::terminal::{input::InputDispatcher, renderer::Renderer};

use message_io::events::{EventSender, EventQueue};

use std::time::{Duration};

lazy_static! {
    static ref APP_FRAME_DURATION: Duration = Duration::from_secs_f32(1.0 / 30.0);
}

#[derive(Debug)]
pub enum AppEvent {
    Action(Action),
    Close,
    Draw,
}

pub struct Application {
    event_queue: EventQueue<AppEvent>,
    store: Store<ActionManager>,
    _server: ServerProxy, // Kept because we need its internal thread running
    _input: InputDispatcher, // Kept because we need its internal thread running
}

impl Application {
    pub fn new(config: Config) -> Application {
        let mut event_queue = EventQueue::new();

        let server_event_sender = event_queue.sender().clone();
        let mut server = ServerProxy::new(move |server_event| {
            server_event_sender.send(AppEvent::Action(Action::ServerEvent(server_event)))
        });

        let state = State::new(config);
        let app_controller = ApplicationController { sender: event_queue.sender().clone() };
        let actions = ActionManager::new(app_controller, server.api());

        let action_dispatcher = ActionDispatcher { sender: event_queue.sender().clone() };

        Application {
            event_queue,
            store: Store::new(state, actions),
            _server: server,
            _input: InputDispatcher::new(action_dispatcher),
        }
    }

    pub fn run(&mut self) {
        self.store.dispatch(Action::StartApp);
        self.event_queue.sender().send(AppEvent::Draw);

        let mut renderer = Renderer::new();
        loop {
            let event = self.event_queue.receive();
            log::trace!("[Process event] - {:?}", event);
            match event {
                AppEvent::Action(action) => {
                    self.store.dispatch(action);
                },
                AppEvent::Draw => {
                    renderer.render(&self.store.state());
                    self.event_queue.sender().send_with_timer(AppEvent::Draw, *APP_FRAME_DURATION);
                },
                AppEvent::Close => {
                    log::info!("Closing client");
                    break
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

pub struct ApplicationController {
    sender: EventSender<AppEvent>
}

impl AppController for ApplicationController {
    fn close(&mut self) {
        self.sender.send_with_priority(AppEvent::Close);
    }
}
