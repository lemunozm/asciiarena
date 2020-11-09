use super::configuration::{Config};
use super::state::{State};
use super::store::{Store, Action, AppController};
use super::server_proxy::{ServerProxy, ServerEvent};

use super::gui::input::{InputReceiver, InputEvent};
use super::gui::renderer::{Renderer};
use super::gui::element::{GuiElement};
use super::gui::elements::gui::{Gui};

use message_io::events::{EventSender, EventQueue};

use std::time::{Duration};

lazy_static! {
    static ref APP_FRAME_DURATION: Duration = Duration::from_secs_f32(1.0 / 30.0);
}

#[derive(Debug)]
pub enum AppEvent {
    ServerEvent(ServerEvent),
    InputEvent(InputEvent),
    Draw,
    Close,
}

pub struct Application {
    event_queue: EventQueue<AppEvent>,
    store: Store,
    gui: Gui,
    server: Option<ServerProxy>,
    input: Option<InputReceiver>,
}

impl Application {
    pub fn new(config: Config) -> Application {
        let mut event_queue = EventQueue::new();

        let event_sender = event_queue.sender().clone();
        let mut server = ServerProxy::new(move |server_event| {
            event_sender.send(AppEvent::ServerEvent(server_event))
        });

        let event_sender = event_queue.sender().clone();
        let input = InputReceiver::new(move |input_event| {
            event_sender.send(AppEvent::InputEvent(input_event))
        });

        let app_controller = ApplicationController { sender: event_queue.sender().clone() };

        Application {
            event_queue,
            store: Store::new(State::new(&config), app_controller, server.api()),
            gui: Gui::new(&config),
            server: Some(server),
            input: Some(input),
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
                AppEvent::ServerEvent(server_event) => {
                    self.store.dispatch(Action::ServerEvent(server_event));
                },
                AppEvent::InputEvent(input_event) => {
                    self.gui.process_event(&mut self.store, input_event);
                },
                AppEvent::Draw => {
                    self.gui.update(self.store.state());
                    renderer.render(self.store.state(), &self.gui);
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

impl Drop for Application {
    fn drop(&mut self) {
        self.server = None; // Server thread stop here (before event_queue drop)
        self.input = None; // Input thread stop here (before event_queue drop)
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
