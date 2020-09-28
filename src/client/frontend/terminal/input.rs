use super::events::{TerminalEventCollector};

use crate::client::actions::{ActionManager, Action};
use crate::client::util::store::{Store};
use crate::client::frontend::{Input as InputBase};

use message_io::events::{Senderable};

use crossterm::event::{Event as TermEvent, KeyEvent, KeyCode, KeyModifiers};

pub struct Input {
    store: Store<ActionManager>,
    _event_collector: TerminalEventCollector, // Keep because we need its internal thread running
}

impl InputBase for Input {
    type InputEvent = TermEvent;

    fn new<S>(store: Store<ActionManager>, input_sender: S) -> Input
    where S: Senderable<Self::InputEvent> + Send + 'static + Clone {
        let _event_collector = TerminalEventCollector::new(move |terminal_event| {
            input_sender.send(terminal_event)
        });

        Input {
            store,
            _event_collector,
        }
    }

    fn process_event(&mut self, event: Self::InputEvent) {
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
