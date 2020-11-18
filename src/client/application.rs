use super::configuration::{Config};
use super::state::{State};
use super::store::{Store, Action, AppController};
use super::server_proxy::{ServerProxy, ServerEvent};

use super::terminal::input::{InputReceiver, InputEvent};
use super::terminal::renderer::{Renderer};
use super::terminal::widgets::gui::{Gui};

use message_io::events::{EventQueue};

use std::time::{Duration};
use std::rc::{Rc};
use std::cell::{Cell};

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
    event_queue: EventQueue<AppEvent>,
    store: Store,
    gui: Gui,
    server: Option<ServerProxy>,
    input: Option<InputReceiver>,
    app_controller: ApplicationController,
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

        let app_controller = ApplicationController::default();

        Application {
            event_queue,
            store: Store::new(State::new(&config), app_controller.clone(), server.api()),
            gui: Gui::new(&config),
            server: Some(server),
            input: Some(input),
            app_controller,
        }
    }

    pub fn run(&mut self) {
        self.store.dispatch(Action::StartApp);
        self.event_queue.sender().send(AppEvent::Draw);

        let mut renderer = Renderer::new();
        loop {
            if self.app_controller.should_close() {
                return log::info!("Closing client");
            }

            let event = self.event_queue.receive();
            match event {
                AppEvent::ServerEvent(server_event) => {
                    log::trace!("[Process server event] - {:?}", server_event);
                    self.store.dispatch(Action::ServerEvent(server_event));
                },
                AppEvent::InputEvent(input_event) => {
                    log::trace!("[Process input event] - {:?}", input_event);
                    self.gui.process_event(&mut self.store, input_event);
                },
                AppEvent::Draw => {
                    self.gui.update(self.store.state());
                    renderer.render(self.store.state(), &self.gui);
                    self.event_queue.sender().send_with_timer(AppEvent::Draw, *APP_FRAME_DURATION);
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

#[derive(Clone)]
pub struct ApplicationController {
    should_close: Rc<Cell<bool>>
}

impl ApplicationController {
    pub fn should_close(&self) -> bool {
        self.should_close.get()
    }
}

impl Default for ApplicationController {
    fn default() -> ApplicationController {
       ApplicationController {
           should_close: Rc::new(Cell::new(false))
       }
    }
}

impl AppController for ApplicationController {
    fn close(&mut self) {
        self.should_close.set(true);
    }
}
