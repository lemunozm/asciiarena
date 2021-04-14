use super::configuration::{Config};
use super::state::{State};
use super::store::{Store, Action};
use super::server_proxy::{ServerProxy, ServerEvent};

use super::terminal::input::{InputReceiver, InputEvent};
use super::terminal::renderer::{Renderer};
use super::terminal::widgets::gui::{Gui};

use message_io::events::{EventReceiver};

use std::time::{Duration};

lazy_static! {
    static ref APP_FRAME_DURATION: Duration = Duration::from_secs_f32(1.0 / 30.0);
}

#[derive(Debug)]
pub enum AppEvent {
    ServerEvent(ServerEvent),
    InputEvent(InputEvent),
    Draw,
}

pub struct Application {
    store: Store,
    gui: Gui,
    _server: ServerProxy,
    _input: InputReceiver,
    event_queue: EventReceiver<AppEvent>,
}

impl Application {
    pub fn new(config: Config) -> Application {
        let mut event_queue = EventReceiver::default();

        let event_sender = event_queue.sender().clone();
        let mut server = ServerProxy::new(move |server_event| {
            event_sender.send(AppEvent::ServerEvent(server_event))
        });

        let event_sender = event_queue.sender().clone();
        let input = InputReceiver::new(move |input_event| {
            event_sender.send(AppEvent::InputEvent(input_event))
        });

        Application {
            store: Store::new(State::new(&config), server.api()),
            gui: Gui::new(&config),
            _server: server,
            _input: input,
            event_queue,
        }
    }

    pub fn run(&mut self) {
        self.store.dispatch(Action::StartApp);
        self.event_queue.sender().send(AppEvent::Draw);

        let mut renderer = Renderer::new();
        loop {
            if self.store.should_close() {
                return log::info!("Closing client")
            }

            let event = self.event_queue.receive();
            match event {
                AppEvent::ServerEvent(server_event) => {
                    log::trace!("[Process server event] - {:?}", server_event);
                    self.store.dispatch(Action::ServerEvent(server_event));
                }
                AppEvent::InputEvent(input_event) => {
                    log::trace!("[Process input event] - {:?}", input_event);
                    self.gui.process_event(&mut self.store, input_event);
                }
                AppEvent::Draw => {
                    self.gui.update(self.store.state());
                    renderer.render(self.store.state(), &self.gui);
                    self.event_queue.sender().send_with_timer(AppEvent::Draw, *APP_FRAME_DURATION);
                }
            }
        }
    }
}
