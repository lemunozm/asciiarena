use super::events::{TerminalEventCollector};

use crate::client::actions::{ActionManager, Action};
use crate::client::util::store::{Store};

use message_io::events::{Senderable};

use crossterm::event::{Event as TermEvent, KeyEvent, KeyCode, KeyModifiers};

pub type InputEvent = TermEvent;

pub struct Input {
    store: Store<ActionManager>,
    event_collector: TerminalEventCollector,
}

impl Input {
    pub fn new<S>(store: Store<ActionManager>, input_sender: S) -> Input
    where S: Senderable<InputEvent> + Send + 'static + Clone {
        let event_collector = TerminalEventCollector::new(move |terminal_event| {
            input_sender.send(terminal_event)
        });

        Input {
            store,
            event_collector,
        }
    }

    pub fn process_event(&mut self, event: InputEvent) {
        match event {
            TermEvent::Key(KeyEvent{code, modifiers}) => match code {
                KeyCode::Esc => {
                    self.store.dispatch(Action::Close);
                },
                KeyCode::Char(character) => {
                    if character == 'c' && modifiers.contains(KeyModifiers::CONTROL) {
                        self.store.dispatch(Action::Close);
                    }
                },
                _ => (),
            }
            _ => (),
        }
    }
}
