use super::server_proxy::{ServerProxy};
use super::util::store::{Store};
use super::actions::{ActionManager, Action, AppController};
use super::state::{State};
pub use super::state::{Config};

use super::terminal::input::{InputReceiver};
use super::terminal::renderer::{Renderer};

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
    _server: ServerProxy, // Kept because we need its internal thread running until drop
    _input: InputReceiver, // Kept because we need its internal thread running until drop
}

impl Application {
    pub fn new(config: Config) -> Application {
        let mut event_queue = EventQueue::new();

        let event_sender = event_queue.sender().clone();
        let mut server = ServerProxy::new(move |server_event| {
            event_sender.send(AppEvent::Action(Action::ServerEvent(server_event)))
        });

        let event_sender = event_queue.sender().clone();
        let input = InputReceiver::new(move |input_event| {
            event_sender.send(AppEvent::Action(Action::InputEvent(input_event)))
        });

        let state = State::new(config);
        let app_controller = ApplicationController { sender: event_queue.sender().clone() };
        let actions = ActionManager::new(app_controller, server.api());

        Application {
            event_queue,
            store: Store::new(state, actions),
            _server: server,
            _input: input,
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

pub struct ApplicationController {
    sender: EventSender<AppEvent>
}

impl AppController for ApplicationController {
    fn close(&mut self) {
        self.sender.send_with_priority(AppEvent::Close);
    }
}
